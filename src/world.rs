use crate::*;

pub struct World {
	world: *mut ecs_world_t,

	// for now this is the simplest way to cache component IDs etc
	// type_map: HashMap<TypeId, u64>,
}

impl World {
	pub fn new() -> Self {
		let world = unsafe { ecs_init() };
		//init_builtin_components();
		Self {
			world,
			// type_map: HashMap::new(),
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
        return unsafe { ecs_progress(self.world, delta_time) }
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

	pub fn get<T: Component>(&self, entity: Entity) -> &T {
		let comp_id = component_id_for_type::<T>();
		let value = unsafe { ecs_get_id(self.world, entity.raw(), comp_id) };
		unsafe { (value as *const T).as_ref().unwrap() }
	}

	pub fn add<T: Component>(self, entity: Entity) -> Self {
        // flecs_static_assert(is_flecs_constructible<T>::value,
        //     "cannot default construct type: add T::T() or use emplace<T>()");
		let comp_id = component_id_for_type::<T>();
        unsafe { ecs_add_id(self.world, entity.raw(), comp_id) };
		self
	}

	pub fn id<T: Component>(&mut self) -> Option<Entity> {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = TYPE_MAP.lock().unwrap().get(&type_id) {
			return Some(Entity::new(*comp_id));
		}
		None
	}

	pub fn component<T: 'static>(&mut self, name: Option<&str>) -> Entity {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = TYPE_MAP.lock().unwrap().get(&type_id) {
			return Entity::new(*comp_id);
		}

		// let result: Entity = pod_component<T>(world, name);
	
		// if (_::cpp_type<T>::size()) {
		// 	_::register_lifecycle_actions<T>(world, result);
		// }
		
		let symbol = std::any::type_name::<T>();
		let layout = std::alloc::Layout::new::<T>();
		let comp_id = register_component(self.world, name, symbol, layout);
		TYPE_MAP.lock().unwrap().insert(type_id, comp_id);
		Entity::new(comp_id)
	}	
}

impl Drop for World {
	fn drop(&mut self) {
		TYPE_MAP.lock().unwrap().clear();
	}
}