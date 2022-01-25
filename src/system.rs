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

// We will merge this into the new SystemBuilder at some point to support iter() impls
pub struct SystemBuilderOld {
	world: *mut ecs_world_t,
	desc: ecs_system_desc_t,

	// we need to keep these in memory until after build
	name_temp: String,	
	signature_temp: String,	
}

impl SystemBuilderOld {
	pub(crate) fn new(world: *mut ecs_world_t) -> Self {
		let mut desc = ecs_system_desc_t::default();

		// m_desc.entity.name = name;
		desc.entity.sep = NAME_SEP.as_ptr() as *const i8;
		desc.entity.add[0] = unsafe { EcsOnUpdate };
		// m_desc.query.filter.expr = expr;
		// this->populate_filter_from_pack();

		SystemBuilderOld {
			world,
			desc,
			name_temp: "".to_owned(),
			signature_temp: "".to_owned(),
		}
	}

    pub fn named(mut self, name: &str) -> Self {
        self.name_temp = name.to_owned();
		self
    }

    pub fn signature(mut self, signature: &str) -> Self {
        self.signature_temp = signature.to_owned();
        self
    }

    pub fn interval(mut self, interval: f32) -> Self {
        self.desc.interval = interval;
		self
    }

	/** Associate system with entity */
	pub fn entity(mut self, entity: Entity) -> Self {
		self.desc.self_ = entity.raw();
		self
	}
	
    /** Set system context */
    pub(crate) fn ctx(mut self, ctx: *mut ::std::os::raw::c_void) -> Self {
        self.desc.ctx = ctx;
        self
    }	

	// Build APIs, the 2 variants call the internal build()
	fn build(&mut self) -> ecs_entity_t {
		let e: ecs_entity_t;

		let name_c_str = std::ffi::CString::new(self.name_temp.as_str()).unwrap();
		self.desc.entity.name = name_c_str.as_ptr() as *const i8;

		let signature_c_str = std::ffi::CString::new(self.signature_temp.as_str()).unwrap();
		if self.signature_temp.len() > 0 {
			self.desc.query.filter.expr = signature_c_str.as_ptr() as *const i8;
		}

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

			e = unsafe { ecs_system_init(self.world, &self.desc) };
        }

        // if (this->m_desc.query.filter.terms_buffer) {
        //     ecs_os_free(m_desc.query.filter.terms_buffer);
        // }

        e
	}

	// temp signature until I get component bundles working
	pub fn iter<F: FnMut(&Iter)>(mut self, mut func: F) -> System {
		// we have to wrap the passed in function in a trampoline
		// so that we can access it again within the C callback handler
		let mut closure = |it: *mut ecs_iter_t| {
			let iter = Iter::new(it);
			func(&iter);
		};
		let trampoline = get_trampoline(&closure);

		self.desc.callback = Some(trampoline);
		self = self.ctx(&mut closure as *mut _ as *mut c_void);

		let e = Self::build(&mut self);
		System::new(self.world, e)
	}

	// each will be similar to above, but with different signature
	// pub fn each<F: FnMut(&Iter)>(mut self, mut func: F) -> System {
	// }
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
	fn new(it: *mut ecs_iter_t) -> Self {
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

    pub fn term<A: Component>(&self, index: i32) -> Column<A> {
        Self::get_term::<A>(self, index)
    }

    fn get_term<T: Component>(&self, index: i32) -> Column<T> {
		// validate that types match. could avoid this in Release builds perhaps to get max perf
        let term_id = unsafe { ecs_term_id(self.it, index) };
		let world = unsafe { (*self.it).real_world };	// must use real to get component infos
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(world).expect("Component type not registered!");
		// println!("Term: {}, Comp: {}", term_id, comp_id);
		assert!(term_id == comp_id);

		/* TODO - validate that the types actually match!!!!
#ifndef NDEBUG
        ecs_assert(term_id & ECS_PAIR || term_id & ECS_SWITCH || 
            term_id & ECS_CASE ||
            term_id == _::cpp_type<T>::id(m_iter->world), 
            ECS_COLUMN_TYPE_MISMATCH, NULL);
#endif
		*/

        let mut count = self.count();

        let is_shared = unsafe { !ecs_term_is_owned2(self.it, index) };

        /* If a shared column is retrieved with 'column', there will only be a
         * single value. Ensure that the application does not accidentally read
         * out of bounds. */
        if is_shared {
            count = 1;
        }
		// println!("Term: {}, is_shared: {}, count: {}", term_id, is_shared, count);

		let size = std::mem::size_of::<T>();
		let array = unsafe { ecs_term_w_size(self.it, size as u64, index) as *mut T };
        
		Column::new(array, count, is_shared)
    }

    pub fn get_term_dynamic(&self, index: i32) -> ColumnDynamic {
        let mut count = self.count();
        let is_shared = unsafe { !ecs_term_is_owned2(self.it, index) };
        if is_shared {
            count = 1;
        }

		// TODO: look this up within the component info
		let world = unsafe { (*self.it).real_world };
        let term_id = unsafe { ecs_term_id(self.it, index) };
		
		let mut size = 0;	// we only get a size if there is a component?
		if let Some(info) = get_component_info(world, term_id) {
			size = info.size;
		}

		let array = unsafe { ecs_term_w_size(self.it, size as u64, index) as *mut u8 };
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

	let ctx = (*it).ctx;
	if ctx.is_null() {
		return;
	}

	let user_data = (*it).ctx;
    let func = &mut *(user_data as *mut F);
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
}


/// V2  with typed component groups
/// 

pub struct SystemBuilder<'c, G: ComponentGroup<'c>> {
	world: &'c World,
	desc: ecs_system_desc_t,

	// we need to keep these in memory until after build
	name_temp: String,	
	signature_temp: String,	

	_phantom: std::marker::PhantomData<G>,
}

impl<'c, G: ComponentGroup<'c>> SystemBuilder<'c, G> {
	pub(crate) fn new(world: &'c World) -> Self {
		let world_raw = world.raw();
		let mut desc = ecs_system_desc_t::default();

		// m_desc.entity.name = name;
		desc.entity.sep = NAME_SEP.as_ptr() as *const i8;
		desc.entity.add[0] = unsafe { EcsOnUpdate };
		// m_desc.query.filter.expr = expr;
		// this->populate_filter_from_pack();

		SystemBuilder {
			world,
			desc,
			name_temp: "".to_owned(),
			signature_temp: "".to_owned(),
			_phantom: Default::default(),
		}
	}

    pub fn named(mut self, name: &str) -> Self {
        self.name_temp = name.to_owned();
		self
    }

    pub fn signature(mut self, signature: &str) -> Self {
        self.signature_temp = signature.to_owned();
        self
    }

    pub fn interval(mut self, interval: f32) -> Self {
        self.desc.interval = interval;
		self
    }

	/** Associate system with entity */
	pub fn entity(mut self, entity: Entity) -> Self {
		self.desc.self_ = entity.raw();
		self
	}
	
    /** Set system context */
    pub(crate) fn ctx(mut self, ctx: *mut ::std::os::raw::c_void) -> Self {
        self.desc.ctx = ctx;
        self
    }	

	// Build APIs, the 2 variants call the internal build()
	fn build(&mut self) -> ecs_entity_t {
		let world_raw = self.world.raw();
		let e: ecs_entity_t;

		let name_c_str = std::ffi::CString::new(self.name_temp.as_str()).unwrap();
		self.desc.entity.name = name_c_str.as_ptr() as *const i8;

		let signature_c_str = std::ffi::CString::new(self.signature_temp.as_str()).unwrap();
		if self.signature_temp.len() > 0 {
			self.desc.query.filter.expr = signature_c_str.as_ptr() as *const i8;
		} else {
			// we should infer some filter state from the <(A, B)> generic signature
		}

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

			e = unsafe { ecs_system_init(world_raw, &self.desc) };
        }

        // if (this->m_desc.query.filter.terms_buffer) {
        //     ecs_os_free(m_desc.query.filter.terms_buffer);
        // }

        e
	}

	pub fn each(mut self, mut cb: impl FnMut(Entity, G::RefTuple)) -> System {
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
		self = self.ctx(&mut closure as *mut _ as *mut c_void);

		let e = Self::build(&mut self);
		System::new(self.world.raw(), e)		
	}

	pub fn each_mut(mut self, mut cb: impl FnMut(Entity, G::MutRefTuple)) -> System {
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
		self = self.ctx(&mut closure as *mut _ as *mut c_void);

		let e = Self::build(&mut self);
		System::new(self.world.raw(), e)		
	}

	// temp signature until I get component bundles working
	pub fn iter<F: FnMut(&Iter)>(mut self, mut func: F) -> System {
		// we have to wrap the passed in function in a trampoline
		// so that we can access it again within the C callback handler
		let mut closure = |it: *mut ecs_iter_t| {
			let iter = Iter::new(it);
			func(&iter);
		};
		let trampoline = get_trampoline(&closure);

		self.desc.callback = Some(trampoline);
		self = self.ctx(&mut closure as *mut _ as *mut c_void);

		let e = Self::build(&mut self);
		System::new(self.world.raw(), e)
	}
}