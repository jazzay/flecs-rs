use crate::*;

pub type EntityId = ecs_entity_t;

// impl From<u64> for EntityId {
//     fn from(v: u64) -> Self {
//         v as EntityId
//     }
// }

// WIP - This should become like the flecs::entity class
//
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Entity {
	entity: EntityId,
	world: *mut ecs_world_t,
}

impl Entity {
	pub(crate) fn new(world: *mut ecs_world_t, entity: EntityId) -> Self {
		Self { entity, world }
	}

	pub(crate) fn raw(&self) -> EntityId { 
		self.entity 
	}

	pub fn id(&self) -> EntityId { 
		self.entity 
	}

	pub fn is_a(self, object: EntityId) -> Self {
        unsafe { self.add_relation(EcsIsA, object) }
	}

	pub fn add_id(self, id: EntityId) -> Self {
        unsafe { ecs_add_id(self.world, self.entity, id) };
		self
	}

	pub fn add_relation(self, relation: EntityId, object: EntityId) -> Self {
        let pair = unsafe { ecs_make_pair(relation, object) };
		self.add_id(pair)
	}

	pub fn get<T: Component>(&self) -> &T {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		let value = unsafe { ecs_get_id(self.world, self.entity, comp_id) };
		unsafe { (value as *const T).as_ref().unwrap() }
	}

	pub fn destruct(self) {
		unsafe { ecs_delete(self.world, self.entity) }; 
	}
}

impl From<Entity> for u64 {
    fn from(e: Entity) -> Self {
        e.raw()
    }
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

	pub fn is_a(self, object: Entity) -> Self {
        unsafe { self.add_relation(EcsIsA, object.raw()) }
	}

	pub fn add_id(self, id: EntityId) -> Self {
        unsafe { ecs_add_id(self.world, self.entity, id) };
		self
	}

	pub fn add_relation(self, relation: EntityId, object: EntityId) -> Self {
        let pair = unsafe { ecs_make_pair(relation, object) };
		self.add_id(pair)
	}

	pub fn set_component(self, comp: EntityId, src: &[u8]) -> Self {
		let info = get_component_info(self.world, comp).expect("Component type not registered!");
		let mut is_added = false;
		let dest = unsafe { 
			let ptr = ecs_get_mut_id(self.world, self.entity, comp, &mut is_added) as *mut u8;
			std::slice::from_raw_parts_mut(ptr, info.size as usize)
		};

		assert!(src.len() == info.size as usize);
		dest.copy_from_slice(src);
		self
	}

	pub fn write_component<F: FnMut(&mut [u8])>(self, comp: EntityId, mut writer: F) -> Self {
		let info = get_component_info(self.world, comp).expect("Component type not registered!");
		let mut is_added = false;
		let dest = unsafe { 
			let ptr = ecs_get_mut_id(self.world, self.entity, comp, &mut is_added) as *mut u8;
			std::slice::from_raw_parts_mut(ptr, info.size as usize)
		};

		writer(dest);
		self
	}

	// Typed Component accessors
	//
    pub fn get_mut<T: Component>(&mut self) -> &mut T  {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		let mut is_added = false;
		let value = unsafe { ecs_get_mut_id(self.world, self.entity, comp_id, &mut is_added) };
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
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, self.entity, comp_id) };
		self
	}

	// Dynamic Component accessors
	//
    fn get_mut_dynamic(&mut self, symbol: &'static str) -> &mut [u8]  {
		let comp_info = WorldInfoCache::get_component_id_for_symbol(self.world, symbol).unwrap();
		let mut is_added = false;
		let value = unsafe { ecs_get_mut_id(self.world, self.entity, comp_info.id, &mut is_added) };
		unsafe { 
			let ptr = value as *mut u8;
			let len = comp_info.size;
			let s = std::slice::from_raw_parts_mut(ptr, len);
			s
		}
    }

	pub fn set_dynamic(mut self, symbol: &'static str, src: &[u8]) -> Self {
		let dest = self.get_mut_dynamic(symbol);
		dest.copy_from_slice(src);
		self
	}

	pub fn add_dynamic(self, symbol: &'static str) -> Self {
		let comp_info = WorldInfoCache::get_component_id_for_symbol(self.world, symbol).unwrap();
        unsafe { ecs_add_id(self.world, self.entity, comp_info.id) };
		self
	}

	// Completing the build
	//
	pub fn build(self) -> Entity {
		Entity::new(self.world, self.entity)
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

	pub fn get_component(&self, comp: EntityId) -> &[u8] {
		let info = get_component_info(self.world, comp).expect("Component type not registered!");
		let src = unsafe { 
			let ptr = ecs_get_id(self.world, self.entity, comp) as *const u8;
			std::slice::from_raw_parts(ptr, info.size as usize)
		};

		assert!(src.len() == info.size as usize);
		src
	}

	pub fn get<T: Component>(&self) -> &T {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
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