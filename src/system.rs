use std::{ffi::c_void};

use crate::*;
use crate::cache::WorldInfoCache;

pub struct System {
	world: *mut ecs_world_t,
	id: ecs_entity_t,
}

impl System {
	pub(crate) fn new(world: *mut ecs_world_t, id: ecs_entity_t) -> Self {
		System {
			world,
			id
		}
	}

	pub fn entity(&self) -> Entity {
		Entity::new(self.world, self.id)
	}

    pub fn interval(&self, interval: f32) {
        unsafe { ecs_set_interval(self.world, self.id, interval) };
    }

    pub fn enable(&self) {
        unsafe { ecs_enable(self.world, self.id, true) };
    }

    pub fn disable(&self) {
        unsafe { ecs_enable(self.world, self.id, false) };
    }

	pub fn run(&self, delta_time: f32) {
		let param: *mut ::std::os::raw::c_void = std::ptr::null_mut();

		unsafe {
			let _last_entity = ecs_run(
				self.world,
				self.id,
				delta_time,
				param,
			);
		}
	}
}

pub struct SystemBuilder<'w> {
	world: &'w World,
	desc: ecs_system_desc_t,

	// we need to keep these in memory until after build
	name_temp: String,	
	expr_temp: String,	

	next_term_index: usize,
}

impl<'w> TermBuilder for SystemBuilder<'w> {
    fn world(&mut self) -> *mut ecs_world_t {
        self.world.raw()
    }

	fn filter_desc(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc.query.filter
	}

    fn current_term(&mut self) -> &mut ecs_term_t {
        &mut self.desc.query.filter.terms[self.next_term_index]
    }

    fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}

impl<'w> SystemBuilder<'w> {
	pub(crate) fn new(world: &'w World) -> Self {
		let world_raw = world.raw();
		let desc = ecs_system_desc_t::default();

		SystemBuilder {
			world,
			desc,
			name_temp: "".to_owned(),
			expr_temp: "".to_owned(),
			next_term_index: 0,
		}
	}

    pub fn named(mut self, name: &str) -> Self {
        self.name_temp = name.to_owned();
		self
    }

    pub fn expr(mut self, expr: &str) -> Self {
        self.expr_temp = expr.to_owned();
        self
    }

    pub fn interval(mut self, interval: f32) -> Self {
        self.desc.interval = interval;
		self
    }

	/** Associate system with entity */
	// TODO - Don't create an entity then in this case (v3.0 change)
	// pub fn entity(mut self, entity: Entity) -> Self {
	// 	self.desc.entity = entity.raw();
	// 	self
	// }
	
    /** Set system context */
    pub(crate) fn ctx(mut self, ctx: *mut ::std::os::raw::c_void) -> Self {
        self.desc.ctx = ctx;
        self
    }	

	// Build APIs, the 2 variants call the internal build()
	fn build(&mut self) -> ecs_entity_t {
		let world = self.world.raw();
		let e: ecs_entity_t;

		let name_c_str = std::ffi::CString::new(self.name_temp.as_str()).unwrap();

		let mut entity_desc: ecs_entity_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };

		if self.name_temp.len() > 0 {
			entity_desc.name = name_c_str.as_ptr() as *const i8;
		} else {
			// We must pass Null to flecs instead of "" otherwise bad stuff happens!
			entity_desc.name = std::ptr::null()
		}

		// We have to add this pair so that the system is part of standard progress stage
		entity_desc.add[0] = unsafe { ecs_pair(EcsDependsOn, EcsOnUpdate) };

		// create a system entity
		self.desc.entity = unsafe { ecs_entity_init(world, &entity_desc) };

		let expr_c_str = std::ffi::CString::new(self.expr_temp.as_str()).unwrap();
		if self.expr_temp.len() > 0 {
			self.desc.query.filter.expr = expr_c_str.as_ptr() as *const i8;
		} else {
			// we should infer some filter state from the <(A, B)> generic signature
		}

		// TODO: Copied from Flecs C++. Cleanup soon!!
		//
        // entity_t e, kind = m_desc.entity.add[0];
        // bool is_trigger = kind == flecs::OnAdd || kind == flecs::OnRemove;

        /*if (is_trigger) {
            ecs_trigger_desc_t desc = {};
            ecs_term_t term = m_desc.query.filter.terms[0];
            if (ecs_term_is_initialized(&term)) {
                desc.term = term;
            } else {
                desc.expr = m_desc.query.filter.expr;
            }

            desc.entity.entity = m_desc.entity.entity;
            desc.events[0] = kind;
            desc.callback = Invoker::run;
            desc.self = m_desc.self;
            desc.ctx = m_desc.ctx;
            desc.binding_ctx = ctx;
            desc.binding_ctx_free = reinterpret_cast<
                ecs_ctx_free_t>(_::free_obj<Invoker>);

            e = ecs_trigger_init(m_world, &desc);
        } else*/ {
            //let desc = self.desc;
            // desc.callback = Some(Invoker::invoke);
            // desc.self = m_desc.self;
            // desc.query.filter.substitute_default = is_each;
            // desc.binding_ctx = ctx;
            // desc.binding_ctx_free = reinterpret_cast<ecs_ctx_free_t>(_::free_obj<Invoker>);

			e = unsafe { ecs_system_init(world, &self.desc) };
        }

        // if (this->m_desc.query.filter.terms_buffer) {
        //     ecs_os_free(m_desc.query.filter.terms_buffer);
        // }

        e
	}

	pub fn each<G: ComponentGroup<'w>>(mut self, mut cb: impl FnMut(Entity, G::RefTuple)) -> System {
		let mut closure = |it: *mut ecs_iter_t| {
			unsafe {
				let it = &(*it);
				for i in 0..it.count {
					let eid = it.entities.offset(i as isize).as_ref().unwrap();
					let e = Entity::new(it.world, *eid);
					let rt = G::iter_as_ref_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		};
		let trampoline = get_trampoline(&closure);

		self.desc.callback = Some(trampoline);
		self.desc.binding_ctx = &mut closure as *mut _ as *mut c_void;

		let e = Self::build(&mut self);
		System::new(self.world.raw(), e)		
	}

	pub fn each_mut<G: ComponentGroup<'w>>(mut self, mut cb: impl FnMut(Entity, G::MutRefTuple)) -> System {
		let mut closure = |it: *mut ecs_iter_t| {
			unsafe {
				let it = &(*it);
				for i in 0..it.count {
					let eid = it.entities.offset(i as isize).as_ref().unwrap();
					let e = Entity::new(it.world, *eid);
					let rt = G::iter_as_mut_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		};
		let trampoline = get_trampoline(&closure);

		self.desc.callback = Some(trampoline);
		self.desc.binding_ctx = &mut closure as *mut _ as *mut c_void;

		let e = Self::build(&mut self);
		System::new(self.world.raw(), e)		
	}

	pub fn iter<F: FnMut(&Iter)>(mut self, mut func: F) -> System {
		// we have to wrap the passed in function in a trampoline
		// so that we can access it again within the C callback handler
		let mut closure = |it: *mut ecs_iter_t| {
			let iter = Iter::new(it);
			func(&iter);
		};
		let trampoline = get_trampoline(&closure);

		self.desc.callback = Some(trampoline);
		self.desc.binding_ctx = &mut closure as *mut _ as *mut c_void;

		let e = Self::build(&mut self);
		System::new(self.world.raw(), e)
	}
}

// TODO: Move this to another file now that it's used for Queries, etc
// 	I tried to do it at one point but ran in to a Rust compiler crash :(
//
pub struct Iter {
	it: *mut ecs_iter_t,
	begin: usize,
	end: usize,
}

impl Iter {
	pub (crate) fn new(it: *mut ecs_iter_t) -> Self {
		Iter {
			it,
			begin: 0,
			end: unsafe { (*it).count as usize }
		}
	}

	pub fn world(&self) -> World {
		World::new_from(unsafe { (*self.it).world })
	}

	pub(crate) fn real_world(&self) -> World {
		World::new_from(unsafe { (*self.it).real_world })
	}

	pub fn system(&self) -> Entity {
		unsafe { Entity::new((*self.it).world, (*self.it).system) }
	}

	pub fn count(&self) -> usize {
		unsafe { (*self.it).count as usize }
	}

	pub fn delta_time(&self) -> f32 {
		unsafe { (*self.it).delta_time }
	}

	pub fn delta_system_time(&self) -> f32 {
		unsafe { (*self.it).delta_system_time }
	}

    pub fn entity(&self, index: i32) -> Entity {
		let entity = unsafe {
			let id = (*self.it).entities.offset(index as isize);
			id.as_ref().unwrap()
		};

		Entity::new(unsafe { (*self.it).world }, *entity)
    }

    pub fn field<A: Component>(&self, index: i32) -> Column<A> {
        Self::get_field::<A>(self, index)
    }

    fn get_field<T: Component>(&self, index: i32) -> Column<T> {
			// validate that types match. could avoid this in Release builds perhaps to get max perf
			let field_id = unsafe { ecs_field_id(self.it, index) };
			let world = unsafe { (*self.it).real_world };	// must use real to get component infos
			let comp_id = WorldInfoCache::get_component_id_for_type::<T>(world).expect("Component type not registered!");
			// println!("Term: {}, Comp: {}", term_id, comp_id);
			assert!(field_id == comp_id);

			/* TODO - validate that the types actually match!!!!
	#ifndef NDEBUG
					ecs_assert(term_id & ECS_PAIR || term_id & ECS_SWITCH || 
							term_id & ECS_CASE ||
							term_id == _::cpp_type<T>::id(m_iter->world), 
							ECS_COLUMN_TYPE_MISMATCH, NULL);
	#endif
			*/

			let mut count = self.count();

			let is_shared = unsafe { !ecs_field_is_self(self.it, index) };

			/* If a shared column is retrieved with 'column', there will only be a
				* single value. Ensure that the application does not accidentally read
				* out of bounds. */
			if is_shared {
					count = 1;
			}
			// println!("Term: {}, is_shared: {}, count: {}", term_id, is_shared, count);

			let size = std::mem::size_of::<T>();
			let array = unsafe { ecs_field_w_size(self.it, size, index) as *mut T };

			Column::new(array, count, is_shared)
    }

    pub fn field_dynamic(&self, index: i32) -> ColumnDynamic {
			let mut count = self.count();

			let is_shared = unsafe { !ecs_field_is_self(self.it, index) };
			if is_shared {
				count = 1;
			}

			// TODO: look this up within the component info
			let world = unsafe { (*self.it).real_world };
				let term_id = unsafe { ecs_field_id(self.it, index) };

			let mut size = 0;	// we only get a size if there is a component?
			if let Some(info) = get_component_info(world, term_id) {
				size = info.size as usize;
			}

			let array = unsafe { ecs_field_w_size(self.it, size, index) as *mut u8 };

			ColumnDynamic::new(array, count, size as usize, is_shared)
    }	
}

pub type SystemCallback = unsafe extern "C" fn(*mut ecs_iter_t);

unsafe extern "C" fn trampoline<F>(it: *mut ecs_iter_t)
where
    F: FnMut(*mut ecs_iter_t),
{
	if it.is_null() {
		return;
	}

	let func_ptr = (*it).binding_ctx;
	if func_ptr.is_null() {
		return;
	}

    let func = &mut *(func_ptr as *mut F);
    func(it);
}

// we have to wrap system callback functions in a trampoline
// so that we can access it again within the C callback handler
fn get_trampoline<F>(_closure: &F) -> SystemCallback
where
    F: FnMut(*mut ecs_iter_t),
{
    trampoline::<F>
}


// TODO: Move to another file

pub struct Column<T: Component> {
    array: *mut T, 
    count: usize,
    is_shared: bool,
}

impl<T: Component> Column<T> {
	pub(crate) fn new(array: *mut T, count: usize, is_shared: bool) -> Self {
		Column {
			array,
			count,
			is_shared,
		}
	}

	pub fn get(&self, index: usize) -> &T {
		assert!(index < self.count);
		assert!(index == 0 || !self.is_shared);
		unsafe {
			let value = self.array.offset(index as isize);
			value.as_ref().unwrap()
		}
	}

	pub fn get_mut(&self, index: usize) -> &mut T {
		assert!(index < self.count);
		assert!(index == 0 || !self.is_shared);
		unsafe {
			let value = self.array.offset(index as isize);
			value.as_mut().unwrap()
		}
	}
}

pub struct ColumnDynamic {
    array: *mut u8, 
    count: usize,
    element_size: usize,
    is_shared: bool,
}

impl ColumnDynamic {
	pub(crate) fn new(array: *mut u8, count: usize, element_size: usize, is_shared: bool) -> Self {
		ColumnDynamic {
			array,
			count,
			element_size,
			is_shared,
		}
	}

	pub fn element_size(&self) -> usize { self.element_size }

	pub fn get(&self, index: usize) -> &[u8] {
		assert!(index < self.count);
		assert!(index == 0 || !self.is_shared);
		unsafe {
			let element_offset = index * self.element_size;
			let ptr = self.array.offset(element_offset as isize);
			let len = self.element_size;
			std::slice::from_raw_parts_mut(ptr, len)
		}
	}

	pub fn get_mut(&mut self, index: usize) -> &mut [u8] {
		assert!(index < self.count);
		assert!(index == 0 || !self.is_shared);
		unsafe {
			let element_offset = index * self.element_size;
			let ptr = self.array.offset(element_offset as isize);
			let len = self.element_size;
			std::slice::from_raw_parts_mut(ptr, len)
		}
	}
}

