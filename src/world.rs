use std::alloc::Layout;

use crate::*;
use crate::cache::WorldInfoCache;

pub struct World {
	world: *mut ecs_world_t,
	owned: bool,
}

impl World {
	/// Creates a new Flecs World instance
	pub fn new() -> Self {
		let world = unsafe { ecs_init() };
		WorldInfoCache::insert(world);
		//init_builtin_components();
		Self {
			world,
			owned: true
		}
	}

	pub(crate) fn new_from(world: *mut ecs_world_t) -> Self {
		Self {
			world,
			owned: false
		}
	}

	pub fn raw(&self) -> *mut ecs_world_t {
		self.world
	}

	pub fn entity(&self) -> Entity {
		let entity = unsafe { ecs_new_id(self.world) };
		Entity::new(self.world, entity)
	}

	pub fn prefab(&self, name: &str) -> Entity {
		unsafe { 
			let entity = ecs_new_id(self.world);
			Entity::new(self.world, entity)
				.named(name)
				.add_id(EcsPrefab)
		}
	}

    pub fn progress(&self, delta_time: f32) -> bool {
        unsafe { ecs_progress(self.world, delta_time) }
    }	

	pub fn delta_time(&self) -> f32 {
		unsafe { 
			let stats = ecs_get_world_info(self.world).as_ref().unwrap();
			stats.delta_time
		}
	}

	/** Signal application should quit.
     * After calling this operation, the next call to progress() returns false.
     */
    pub fn quit(&self) {
        unsafe { ecs_quit(self.world) }
    }

    /** Test if quit() has been called.
     */
    fn should_quit(&self) -> bool {
        unsafe { ecs_should_quit(self.world) }
    }

	pub fn find_entity(&self, entity: EntityId) -> Option<Entity> {
		// TODO: check that it exists!
		Some(Entity::new(self.world, entity))
	}

	pub fn lookup(&self, name: &str) -> Option<Entity> {
		None
	}

	pub fn name(&self, entity: Entity) -> &str {
		let name_str = unsafe { ecs_get_name(self.world, entity.raw()) };
		unsafe { flecs_to_rust_str(name_str) }
	}

	/// Set a singleton component
	pub fn set_singleton<T: Component>(&mut self, value: T) {
		// insert the singleton type automatically if necessary
		if self.id::<T>().is_none() {
			self.component::<T>();
		}

		let comp_id = self.id::<T>().unwrap();
		let entity = comp_id.clone();	// entity = the component for singleton
		self.set(entity, value);
	}

	/// Get a singleton component mutably
	pub fn get_singleton_mut<'a, T: Component>(&'a mut self) -> Option<&'a mut T> {
		// insert the singleton type automatically if necessary
		if self.id::<T>().is_none() {
			self.component::<T>();
		}

		let comp_id = self.id::<T>().unwrap();
		let entity = comp_id.clone();	// entity = the component for singleton

		let mut is_added = false;
		let dest = unsafe { ecs_get_mut_id(self.world, entity.raw(), comp_id.raw(), &mut is_added) } ;

		if dest.is_null() {
			return None;
		}
		Some(unsafe { (dest as *mut T).as_mut().unwrap() })
	}
	
	/// Get a singleton component 
	pub fn get_singleton<'a, T: Component>(&'a self) -> Option<&'a T> {
		let comp = self.id::<T>().expect("singleton entity does not exist");
		let entity = comp.clone();	// entity = the component for singleton
		self.get_internal::<T>(entity, comp.raw())
	}
	
	// TODO: should we make this return an option over panicing?
	pub fn get<'a, T: Component>(&'a self, entity: Entity) -> Option<&'a T> {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		self.get_internal::<T>(entity, comp_id)
	}

	fn get_internal<'a, T: Component>(&'a self, entity: Entity, comp: u64) -> Option<&'a T> {
		let value = unsafe { ecs_get_id(self.world, entity.raw(), comp) };
		if value.is_null() {
			return None;
		}
		Some(unsafe { (value as *const T).as_ref().unwrap() })
	}

	pub fn add<T: Component>(&self, entity: Entity) {
        // flecs_static_assert(is_flecs_constructible<T>::value,
        //     "cannot default construct type: add T::T() or use emplace<T>()");
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, entity.raw(), comp_id) };
	}

	pub fn set<T: Component>(&self, entity: Entity, value: T) {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		let mut is_added = false;
		let dest = unsafe { ecs_get_mut_id(self.world, entity.raw(), comp_id, &mut is_added) } ;
		let dest = unsafe { (dest as *mut T).as_mut().unwrap() };
		*dest = value;
	}

	pub fn read_component(&self, entity: EntityId, comp: EntityId) -> Option<&[u8]> {
		let info = get_component_info(self.world, comp).expect("Component type not registered!");
		let src = unsafe { 
			let ptr = ecs_get_id(self.world, entity, comp) as *const u8;
			if ptr.is_null() {
				return None;
			}
			std::slice::from_raw_parts(ptr, info.size as usize)
		};

		assert!(src.len() == info.size as usize);
		Some(src)
	}

	pub fn write_component<F: FnMut(&mut [u8])>(&self, entity: EntityId, comp: EntityId, mut writer: F) {
		let info = get_component_info(self.world, comp).expect("Component type not registered!");
		let mut is_added = false;
		let dest = unsafe { 
			let ptr = ecs_get_mut_id(self.world, entity, comp, &mut is_added) as *mut u8;
			std::slice::from_raw_parts_mut(ptr, info.size as usize)
		};

		writer(dest);
	}

	pub fn id<T: Component>(&self) -> Option<Entity> {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = WorldInfoCache::get_component_id_for_type::<T>(self.world) {
			return Some(Entity::new(self.world, comp_id));
		}
		None
	}

    pub fn component_id<T: Component>(&mut self) -> u64  {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		comp_id
	}

	pub fn component<T: 'static>(&mut self) -> Entity {
		let comp_id = register_component_typed::<T>(self.world, None);
		Entity::new(self.world, comp_id)
	}

	pub fn component_named<T: 'static>(&mut self, name: &str) -> EntityId {
		register_component_typed::<T>(self.world, Some(name))
	}

	pub fn component_dynamic(&mut self, symbol: &'static str, layout: Layout) -> EntityId {
		register_component_dynamic(self.world, symbol, None, layout)
	}

	pub fn component_dynamic_named(&mut self, symbol: &'static str, name: &'static str, layout: Layout) -> EntityId {
		register_component_dynamic(self.world, symbol, Some(name), layout)
	}

	pub fn system<'a, G: ComponentGroup<'a>>(&'a self) -> SystemBuilder<'a, G> {
		let sb: SystemBuilder<'a, G> = SystemBuilder::new(self);
        sb
    }	

	// not sure yet if 'dynamic' is the right terminology, basically a system with no generic compile time types.
	//	term T's must be determined explicitly in system implementations and match expression syntax.
	//	Will not support each() calls for now
	pub fn system_dynamic<'a>(&'a self) -> SystemBuilder<'a, ((), ())> {
		let sb: SystemBuilder<'a, ((), ())> = SystemBuilder::new(self);
        sb
    }	

	pub fn filter<'a, G: ComponentGroup<'a>>(&'a self) -> FilterGroup<'a, G> {
		let filter: FilterGroup<'a, G> = FilterGroup::new(self);
        filter
    }	

	// Iterate through all entities matching 1 component
	pub fn each1<A: Component>(&self, mut cb: impl FnMut(Entity, &A)) {
		let filter = Filter::new_1::<A>(self.raw());
		filter.each_1(|e: Entity, a: &A| {
			cb(e, a);
		});
	}

	// Rust compiler will not let is use these short forms, perhaps we can solve the errors
	//
	// pub fn each<'a, G: ComponentGroup<'a>>(&'a self, cb: impl FnMut(Entity, G::RefTuple)) {
	// 	let filter: FilterGroup<'a, G> = FilterGroup::new(self);
	// 	filter.each(cb);
    // }	

	// pub fn each_mut<'a, G: ComponentGroup<'a>>(&'a self, cb: impl FnMut(Entity, G::MutRefTuple)) {
	// 	let filter: FilterGroup<'a, G> = FilterGroup::new(self);
	// 	filter.each_mut(cb);
    // }	

	// But we can easily create these few 'overloads' in terms of component groups. 
	// TODO: Possibly a macro could generate these

	// Iterate through all entities matching 2 components
	pub fn each_2<'a, A: Component, B: Component>(&'a self, mut cb: impl FnMut(Entity, &A, &B)) {
		let filter: FilterGroup<'a, (A, B)> = FilterGroup::new(self);
		filter.each(|e: Entity, (a, b)| {
			cb(e, a, b);
		});
	}

	// Iterate through all entities matching 3 components
	pub fn each_3<'a, A: Component, B: Component, C: Component>(&'a self, mut cb: impl FnMut(Entity, &A, &B, &C)) {
		let filter: FilterGroup<'a, (A, B, C)> = FilterGroup::new(self);
		filter.each(|e: Entity, (a, b, c)| {
			cb(e, a, b, c);
		});
	}

	// Iterate through all entities matching 2 components
	pub fn each_mut_2<'a, A: Component, B: Component>(&'a self, mut cb: impl FnMut(Entity, &mut A, &mut B)) {
		let filter: FilterGroup<'a, (A, B)> = FilterGroup::new(self);
		filter.each_mut(|e: Entity, (a, b)| {
			cb(e, a, b);
		});
	}

	// Iterate through all entities matching 3 components
	pub fn each_mut_3<'a, A: Component, B: Component, C: Component>(&'a self, mut cb: impl FnMut(Entity, &mut A, &mut B, &mut C)) {
		let filter: FilterGroup<'a, (A, B, C)> = FilterGroup::new(self);
		filter.each_mut(|e: Entity, (a, b, c)| {
			cb(e, a, b, c);
		});
	}

}

impl Drop for World {
	fn drop(&mut self) {
		unsafe {
			if self.owned && ecs_stage_is_async(self.world) {
				ecs_async_stage_free(self.world);
			} else if self.owned && !self.world.is_null() {
				ecs_fini(self.world);
			}
		}
	}
}

// Additional Add-ons support
impl World {
	pub fn enable_rest(&self) {
		let rest_comp_id = unsafe { FLECS__EEcsRest as u64 };
    #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
    let rest_comp_size = std::mem::size_of::<EcsRest>() as u32;
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "emscripten")))] 
    let rest_comp_size = std::mem::size_of::<EcsRest>() as u64;
		
		let rest_data: EcsRest = unsafe { MaybeUninit::zeroed().assume_init() };

		unsafe { 
			ecs_set_id(self.raw(), 
				0, 
				rest_comp_id, 
				rest_comp_size, 
				&rest_data as *const EcsRest as *const ::std::os::raw::c_void) 
		};
	}
}