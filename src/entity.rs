use crate::*;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Entity {
	entity: ecs_entity_t,
	// world: *mut ecs_world_t,
}

impl Entity {
	pub(crate) fn new(entity: ecs_entity_t) -> Self {
		Self { entity }
	}

	pub(crate) fn raw(&self) -> ecs_entity_t { self.entity }
}


// explore using the builder pattern to construct Entities with components
//
pub struct EntityBuilder {
	entity: ecs_entity_t,
	world: *mut ecs_world_t,
}

impl EntityBuilder {
	pub fn new(world: *mut ecs_world_t) -> Self {
		let entity = unsafe { ecs_new_id(world) };
		Self { entity, world }
	}

	pub fn name(self, name: &str) -> Self {
		// todo: set the name!
		self
	}

	// private helper
    fn get_mut<T: Component>(&mut self) -> &mut T  {
		let comp_id = WorldInfoCache::component_id_for_type::<T>(self.world);
		let mut is_added = false;
		let value = unsafe { ecs_get_mut_w_entity(self.world, self.entity, comp_id, &mut is_added) };
		unsafe { (value as *mut T).as_mut().unwrap() }
    }

	pub fn set<T: Component>(mut self, value: T) -> Self {
		let dest = self.get_mut::<T>();
		*dest = value;
		self
	}

	pub fn add<T: Component>(self) -> Self {
        // flecs_static_assert(is_flecs_constructible<T>::value,
        //     "cannot default construct type: add T::T() or use emplace<T>()");
		let comp_id = WorldInfoCache::component_id_for_type::<T>(self.world);
        unsafe { ecs_add_id(self.world, self.entity, comp_id) };
		self
	}

	pub fn build(self) -> Entity {
		Entity::new(self.entity)
	}
}

// Read only accessor
#[derive(PartialEq, Eq, Debug)]
pub struct EntityRef {
	entity: ecs_entity_t,
	world: *mut ecs_world_t,
}

impl EntityRef {
	pub(crate) fn new(entity: ecs_entity_t, world: *mut ecs_world_t) -> Self {
		Self { entity, world }
	}

	pub fn name(&self) -> &str {
		let char_ptr = unsafe { ecs_get_name(self.world, self.entity) };
		if char_ptr.is_null() {
			return "";
		}

		let c_str = unsafe { std::ffi::CStr::from_ptr(char_ptr) };
		let name = c_str.to_str().unwrap();
		name
	}

	pub fn get<T: Component>(&self) -> &T {
		let comp_id = WorldInfoCache::component_id_for_type::<T>(self.world);
		let value = unsafe { ecs_get_id(self.world, self.entity, comp_id) };
		unsafe { (value as *const T).as_ref().unwrap() }
	}
}

impl Default for ecs_entity_desc_t {
    fn default() -> Self {
		let desc: ecs_entity_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
		desc
    }
}