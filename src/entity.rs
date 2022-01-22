use std::ptr::slice_from_raw_parts;

use crate::*;

pub type EntityId = ecs_entity_t;

// TODO - Placeholder for now
pub struct EntityTypeInfo {
    entity: Entity,
    table: *const ecs_table_t,
}

impl EntityTypeInfo {
    pub fn new(world: *mut ecs_world_t, entity: EntityId) -> EntityTypeInfo { 
		let table = unsafe { ecs_get_table(world, entity) };

		// type is attached to global entity (0)??
        let entity = Entity::new(world, 0);
		
		EntityTypeInfo {
			entity,
			table
		}
	}

	pub fn to_str(&self) -> &str {
		unsafe {
			let w = ecs_get_world(self.entity.world as *const ecs_poly_t);
			let t = ecs_table_get_type(self.table);
			let type_str = ecs_type_str(w, t);

			// for some reason this str is coming back with weird numeric encoding
			// causing the CStr conversion below to panic. for now return ""
			let type_str = std::ffi::CStr::from_ptr(type_str);
			println!("type_str: {:?}", type_str);

			let type_str = type_str.to_str().unwrap();
			type_str
			// ""
		}
	}
}

// WIP - This should become like the flecs::entity class
//
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Entity {
	entity: EntityId,	// todo: rename this id
	world: *mut ecs_world_t,
}

impl AsEcsId for Entity {
	fn id(&self) -> ecs_id_t {
		self.id()
	}
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

    pub fn is_valid(&self) -> bool {
        !self.world.is_null() && unsafe { ecs_is_valid(self.world, self.entity) }
    }

	// from base id type, which don't exist in rust
    pub fn id_str(&self) -> &str {
		let id_str = unsafe { ecs_id_str(self.world, self.entity) };
		if id_str.is_null() {
			return "";
		}

		let id_str = unsafe { std::ffi::CStr::from_ptr(id_str) };
		if let Ok(id_str) = id_str.to_str() {
			return id_str;
		}

		// TODO - Flecs is returning invalid utf8 strings in some cases
		"Error"
    }

	pub fn name(&self) -> &str {
		if !self.is_valid() {
			return "INVALID";
		}

		let char_ptr = unsafe { ecs_get_name(self.world, self.entity) };
		if char_ptr.is_null() {
			return "";
		}

		let c_str = unsafe { std::ffi::CStr::from_ptr(char_ptr) };
		if let Ok(name) = c_str.to_str() {
			return name;
		}

		// TODO - Flecs is returning invalid utf8 strings in some cases
		// this is due to not having a proper Name assigned generally
		"Error"
	}

	pub fn symbol(&self) -> &str {
		if !self.is_valid() {
			return "INVALID";
		}

		let char_ptr = unsafe { ecs_get_symbol(self.world, self.entity) };
		if char_ptr.is_null() {
			return "";
		}

		// We should always have a proper symbol string
		let c_str = unsafe { std::ffi::CStr::from_ptr(char_ptr) };
		c_str.to_str().unwrap()
	}

	pub fn path(&self) -> &str {
		let sep = NAME_SEP.as_ptr() as *const i8;	// for now only support :: as sep
		let path_ptr = unsafe { ecs_get_path_w_sep(self.world, 0, self.entity, sep, sep) };
		if path_ptr.is_null() {
			return "";
		}

		let c_str = unsafe { std::ffi::CStr::from_ptr(path_ptr) };
		let path = c_str.to_str().unwrap();
		path
	}

	pub fn type_info(&self) -> EntityTypeInfo {
		EntityTypeInfo::new(self.world, self.id())
	}

	pub fn named(self, name: &str) -> Self {
        unsafe { 
			let name_c_str = std::ffi::CString::new(name).unwrap();
			ecs_set_name(self.world, self.entity, name_c_str.as_ptr());
		};
		self
	}

	pub fn is_a<T: AsEcsId>(self, object: T) -> Self {
        unsafe { self.add_relation_ids(EcsIsA, object.id()) }
	}

	pub fn child_of<T: AsEcsId>(self, object: T) -> Self {
        unsafe { self.add_relation_ids(EcsChildOf, object.id()) }
	}

    pub fn has_id<T: AsEcsId>(self, id: T) -> bool {
        unsafe { ecs_has_id(self.world, self.entity, id.id()) }
    }

	pub fn add_id<T: AsEcsId>(self, id: T) -> Self {
        unsafe { ecs_add_id(self.world, self.entity, id.id()) };
		self
	}

    pub fn has_relation<R: AsEcsId, O: AsEcsId>(self, relation: R, object: O) -> bool {
        let pair = unsafe { ecs_make_pair(relation.id(), object.id()) };
        unsafe { ecs_has_id(self.world, self.entity, pair) }
    }

    pub fn has_relation_wildcard<R: AsEcsId>(self, relation: R) -> bool {
        let pair = unsafe { ecs_make_pair(relation.id(), EcsWildcard) };
        unsafe { ecs_has_id(self.world, self.entity, pair) }
    }

    pub fn has_child<T: AsEcsId>(self, child: T) -> bool {
        let pair = unsafe { ecs_make_pair(EcsChildOf, child.id()) };
        unsafe { ecs_has_id(self.world, self.entity, pair) }
    }

	pub fn add_relation_ids<R: AsEcsId, O: AsEcsId>(self, relation: R, object: O) -> Self {
        let pair = unsafe { ecs_make_pair(relation.id(), object.id()) };
		self.add_id(pair)
	}

	pub fn get<T: Component>(&self) -> &T {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		let value = unsafe { ecs_get_id(self.world, self.entity, comp_id) };
		unsafe { (value as *const T).as_ref().unwrap() }
	}

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
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, self.entity, comp_id) };
		self
	}

	pub fn override_component<T: Component>(self) -> Self {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, self.entity, ECS_OVERRIDE | comp_id) };
		self
	}

	pub fn add_relation<R: Component, O: Component>(self) -> Self {
		let relation = WorldInfoCache::get_component_id_for_type::<R>(self.world).expect("Relation type not registered!");
		let object = WorldInfoCache::get_component_id_for_type::<O>(self.world).expect("Object type not registered!");
        let pair = unsafe { ecs_make_pair(relation, object) };
        unsafe { ecs_add_id(self.world, self.entity, pair) };
		self
	}

	pub fn remove<T: Component>(self) -> Self {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_remove_id(self.world, self.entity, comp_id) };
		self
	}

	// Dynamic Components
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

	/// Call this to remove the entity from the world
	pub fn destruct(self) {
		unsafe { ecs_delete(self.world, self.entity) }; 
	}

	pub fn each(&self, mut cb: impl FnMut(Id)) {
		unsafe {
			let e_type = ecs_get_type(self.world, self.entity);
			if e_type.is_null() {
				return;
			}
		
			// let elem_size = std::mem::size_of::<ecs_id_t>() as i32;
			// let elem_align = std::mem::align_of::<ecs_id_t>() as i16;

			let ids = ecs_vector_first::<ecs_id_t>(e_type);
			let count = ecs_vector_count(e_type);

			//println!("got {} ids. size: {}, align: {}", count, elem_size, elem_align);
			let ids = slice_from_raw_parts(ids, count as usize).as_ref().unwrap();
			
			for id in ids {
				//let id = ids[i];
				let id = Id::new(self.world, *id);
				cb(id); 
		
				// TODO: Handle this soon!
				// Case is not stored in type, so handle separately
				// if ((id & ECS_ROLE_MASK) == flecs::Switch) {
				// 	ent = flecs::id(
				// 		m_world, flecs::Case | ecs_get_case(
				// 				m_world, m_id, ent.object().id()));
				// 	func(ent);
				// }
			}
		}
	}

	pub fn children(&self, mut cb: impl FnMut(Entity)) {
		unsafe {
			let mut desc: ecs_filter_desc_t = MaybeUninit::zeroed().assume_init();
			desc.terms[0].id = ecs_make_pair(EcsChildOf, self.id());
			desc.terms[1].id = EcsPrefab;
			desc.terms[1].oper = ecs_oper_kind_t_EcsOptional;

			let mut filter: ecs_filter_t = MaybeUninit::zeroed().assume_init();
			ecs_filter_init(self.world, &mut filter, &desc);

			let mut it = ecs_filter_iter(self.world, &filter);
			while ecs_filter_next(&mut it) {
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(self.world, *eid);
					cb(e);
				}
			}
		}
	}
}

impl From<Entity> for u64 {
    fn from(e: Entity) -> Self {
        e.raw()
    }
}
