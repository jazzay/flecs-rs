use std::ptr::slice_from_raw_parts;

use crate::*;
use crate::cache::WorldInfoCache;

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
			flecs_to_rust_str(type_str)
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
		unsafe { flecs_to_rust_str(id_str) }
    }

	pub fn name(&self) -> &str {
		if !self.is_valid() {
			return "INVALID";
		}

		let name_str = unsafe { ecs_get_name(self.world, self.entity) };
		unsafe { flecs_to_rust_str(name_str) }
	}

	pub fn symbol(&self) -> &str {
		if !self.is_valid() {
			return "INVALID";
		}

		let symbol_str = unsafe { ecs_get_symbol(self.world, self.entity) };
		unsafe { flecs_to_rust_str(symbol_str) }
	}

	pub fn path(&self) -> &str {
		let sep = NAME_SEP.as_ptr() as *const i8;	// for now only support :: as sep
		let path_str = unsafe { ecs_get_path_w_sep(self.world, 0, self.entity, sep, sep) };
		unsafe { flecs_to_rust_str(path_str) }
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

    pub fn is_child_of<T: AsEcsId>(self, parent: T) -> bool {
        let pair = unsafe { ecs_make_pair(EcsChildOf, parent.id()) };
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
		let value = unsafe { ecs_get_mut_id(self.world, self.entity, comp_id) };
		unsafe { (value as *mut T).as_mut().unwrap() }
    }

	pub fn set<T: Component>(mut self, value: T) -> Self {
		let dest = self.get_mut::<T>();
		*dest = value;
		self
	}

	pub fn override_component<T: Component>(self) -> Self {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, self.entity, ECS_OVERRIDE | comp_id) };
		self
	}

	pub fn override_id(self, comp_id: ecs_id_t) -> Self {
        unsafe { ecs_add_id(self.world, self.entity, ECS_OVERRIDE | comp_id) };
		self
	}

	pub fn set_override<T: Component>(self, value: T) -> Self {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		self.override_id(comp_id);
		self.set(value);
		self
	}

	// Added to assess performance impact of Type lookup within Benchmarks
    pub fn set_fast<T: Component>(&self, comp_id: u64, value: T)  {
		// let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
		let ptr = unsafe { ecs_get_mut_id(self.world, self.entity, comp_id) };
		let dest = unsafe { (ptr as *mut T).as_mut().unwrap() };
		*dest = value;
    }

	pub fn add<T: Component>(self) -> Self {
		let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world).expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, self.entity, comp_id) };
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

	// Added to assess performance impact of Type lookup within Benchmarks
	pub fn remove_id(&self, comp_id: u64) {
        unsafe { ecs_remove_id(self.world, self.entity, comp_id) };
	}

	// Dynamic Components
	//
    fn get_mut_dynamic(&mut self, symbol: &'static str) -> &mut [u8]  {
		let comp_info = WorldInfoCache::get_component_id_for_symbol(self.world, symbol).unwrap();
		let value = unsafe { ecs_get_mut_id(self.world, self.entity, comp_info.id) };
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
			let e_type: *const ecs_type_t = ecs_get_type(self.world, self.entity);
			if let Some(ty) = e_type.as_ref() {
				//println!("got {} ids. size: {}, align: {}", count, elem_size, elem_align);
				let ids = slice_from_raw_parts(ty.array, ty.count as usize).as_ref().unwrap();
				
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
	}

	pub fn children(&self, mut cb: impl FnMut(Entity)) {
		unsafe {
			let mut desc: ecs_filter_desc_t = MaybeUninit::zeroed().assume_init();
			desc.terms[0].id = ecs_make_pair(EcsChildOf, self.id());
			desc.terms[1].id = EcsPrefab;
			desc.terms[1].oper = ecs_oper_kind_t_EcsOptional;

			let filter = ecs_filter_init(self.world, &desc);

			let mut it = ecs_filter_iter(self.world, filter);
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
