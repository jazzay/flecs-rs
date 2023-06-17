use once_cell::sync::Lazy;

use crate::bindings::*;
use crate::Component;
use std::{any::TypeId, collections::HashMap, sync::Mutex};

// TODO: Revisit how we cache the runtime Component IDs per type
//
// See if we can find a more efficient solution
//
// 	possibly we could use world->set_context to hold our custom data container
// 	associated to each world, then inside there cache the comp ids, etc
// 	need to watch for mutable vs readonly worlds
//
// Sadly Rust cannot achieve global generic statics like you can in C++
// https://internals.rust-lang.org/t/generic-type-dependent-static-data/8602

// This might help
// https://docs.rs/generic_static/0.2.0/generic_static/
// Issue with statics however is then we don't get per world comp ids
// and registering is done on the world... we would have to follow the
// C++ impl and detect that component was already registered prior and
// assume that same ID again...

static WORLD_INFOS: Lazy<Mutex<HashMap<WorldKey, WorldInfoCache>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

type WorldKey = u64;	//*mut ecs_world_t;

#[derive(Copy, Clone, Debug)]
pub(crate) struct ComponentInfo {
	pub(crate) id: u64,
	pub(crate) size: usize,
}

pub(crate) struct WorldInfoCache
{
	component_typeid_map: HashMap<TypeId, u64>,
	component_symbol_map: HashMap<&'static str, ComponentInfo>,
}

impl WorldInfoCache {
	pub(crate) fn insert(world: *mut ecs_world_t) {
		let cache = WorldInfoCache {
			component_typeid_map: HashMap::new(),
			component_symbol_map: HashMap::new(),
		};

		let world_key = Self::key_for_world(world);
		let mut m = WORLD_INFOS.lock().unwrap();
		m.insert(world_key, cache);
	}

	fn key_for_world(world: *mut ecs_world_t) -> u64 {
		assert!(world != std::ptr::null_mut());

		// Note: flecs dupes the world for execution within systems to prevent
		// writing to the real world, so all mutable operations are deferred. This
		// results in multiple worlds so we must lookup the actual world via ecs_get_world(m_world)
		// so we can gain access to our cached component IDs.
		let actual_world = unsafe { ecs_get_world(world as *const ecs_poly_t) };
		actual_world as u64
	}

	pub fn get_component_id_for_type<T: Component>(world: *mut ecs_world_t) -> Option<ecs_entity_t> {
		let world_key = Self::key_for_world(world);
		let m = WORLD_INFOS.lock().unwrap();
		let cache = m.get(&world_key).unwrap();	//.clone();	

		let type_id = TypeId::of::<T>();
		let comp_id = cache.component_typeid_map.get(&type_id).map(|v| *v);	
		comp_id
	}

	pub fn register_component_id_for_type_id(world: *mut ecs_world_t, comp_id: ecs_entity_t, type_id: TypeId) {
		let world_key = Self::key_for_world(world);
		let mut m = WORLD_INFOS.lock().unwrap();
		let cache = m.get_mut(&world_key).unwrap();	//.clone();	

		cache.component_typeid_map.insert(type_id, comp_id);
	}

	pub fn get_component_id_for_symbol(world: *mut ecs_world_t, symbol: &'static str) -> Option<ComponentInfo> {
		let world_key = Self::key_for_world(world);
		let m = WORLD_INFOS.lock().unwrap();
		let cache = m.get(&world_key).unwrap();
		cache.component_symbol_map.get(symbol).map(|v| *v)
	}

	pub fn register_component_id_for_symbol(world: *mut ecs_world_t, comp_id: ecs_entity_t, symbol: &'static str, size: usize) {
		let world_key = Self::key_for_world(world);
		let mut m = WORLD_INFOS.lock().unwrap();
		let cache = m.get_mut(&world_key).unwrap();
		cache.component_symbol_map.insert(symbol, ComponentInfo { id: comp_id, size });
	}	
}	
