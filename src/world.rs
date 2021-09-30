use std::alloc::Layout;

use crate::*;

pub struct World {
	world: *mut ecs_world_t,
}

impl World {
	pub fn new() -> Self {
		let world = unsafe { ecs_init() };
		WorldInfoCache::insert(world);
		//init_builtin_components();
		Self {
			world,
		}
	}

	pub(crate) fn new_from(world: *mut ecs_world_t) -> Self {
		Self {
			world,
		}
	}

	pub fn raw(&self) -> *mut ecs_world_t {
		self.world
	}

	pub fn entity(&mut self) -> Entity {
		let entity = unsafe { ecs_new_id(self.world) };
		Entity::new(entity)
	}

	pub fn entity_builder(&mut self) -> EntityBuilder {
		EntityBuilder::new(self.world)
	}	

    pub fn progress(&self, delta_time: f32) -> bool {
        unsafe { ecs_progress(self.world, delta_time) }
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

	pub fn lookup(name: &str) -> Option<Entity> {
		None
	}

	pub fn name(&self, entity: Entity) -> &str {
		let char_ptr = unsafe { ecs_get_name(self.world, entity.raw()) };
		let c_str = unsafe { std::ffi::CStr::from_ptr(char_ptr) };
		let name = c_str.to_str().unwrap();
		println!("name(): {}", name);
		name
	}

	// TODO: should we make this return an option over panicing?
	pub fn get<T: Component>(&self, entity: Entity) -> &T {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		let value = unsafe { ecs_get_id(self.world, entity.raw(), comp_id) };
		unsafe { (value as *const T).as_ref().unwrap() }
	}

	pub fn add<T: Component>(self, entity: Entity) -> Self {
        // flecs_static_assert(is_flecs_constructible<T>::value,
        //     "cannot default construct type: add T::T() or use emplace<T>()");
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, entity.raw(), comp_id) };
		self
	}

	pub fn id<T: Component>(&mut self) -> Option<Entity> {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = WorldInfoCache::get_component_id_for_type::<T>(self.world) {
			return Some(Entity::new(comp_id));
		}
		None
	}

	pub fn component<T: 'static>(&mut self) -> Entity {
		register_component_typed::<T>(self.world, None)
	}

	pub fn component_named<T: 'static>(&mut self, name: &str) -> Entity {
		register_component_typed::<T>(self.world, Some(name))
	}

	pub fn component_dynamic(&mut self, symbol: &'static str, layout: Layout) -> Entity {
		register_component_dynamic(self.world, symbol, None, layout)
	}

	pub fn component_dynamic_named(&mut self, symbol: &'static str, name: &str, layout: Layout) -> Entity {
		register_component_dynamic(self.world, symbol, Some(name), layout)
	}

	pub fn system(&mut self) -> SystemBuilder {
		let system = SystemBuilder::new(self.world);
		system
	}
}

impl Drop for World {
	fn drop(&mut self) {
		unsafe { ecs_fini(self.world) };
	}
}