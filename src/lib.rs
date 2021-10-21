#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

use std::{any::TypeId, collections::HashMap, mem::{MaybeUninit}, sync::Mutex};

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
mod bindings;
pub use bindings::*;

mod component;
pub use component::*;

mod entity;
pub use entity::*;

pub mod filter;
pub use filter::*;

pub mod system;
pub use system::*;

pub mod world;
pub use world::*;

// Impl some flecs funcs that were changed to Macros :(

pub unsafe fn ecs_term_id(it: *const ecs_iter_t, index: i32) -> ecs_id_t {
	assert!(index > 0);		// TODO: later add max check as well
	let index = (index - 1) as usize;
	let term_id = (*it).ids.add(index);
	*term_id
}

pub unsafe fn ecs_term_source(it: *const ecs_iter_t, index: i32) -> ecs_entity_t {
	assert!(index > 0);		// TODO: later add max check as well
    if (*it).subjects.is_null() {
		0
	} else {
		let index = (index - 1) as usize;
		*((*it).subjects.add(index))
	} 
}

pub unsafe fn ecs_term_size(it: *const ecs_iter_t, index: i32) -> size_t {
	assert!(index > 0);		// TODO: later add max check as well
    *((*it).sizes.add((index - 1) as usize)) as size_t
}

pub unsafe fn ecs_term_is_owned(it: *const ecs_iter_t, index: i32) -> bool {
	assert!(index > 0);		// TODO: later add max check as well
	let index = (index - 1) as usize;
    (*it).subjects.is_null() || *((*it).subjects.add(index)) == 0
}

// This access query/filter term component data
pub unsafe fn ecs_term<T: Component>(it: *const ecs_iter_t, index: i32) -> *mut T {
	let size = std::mem::size_of::<T>();
	ecs_term_w_size(it, size as size_t, index) as *mut T
}

// This accesses all table columns for a matched archetype
pub unsafe fn ecs_iter_column<T: Component>(it: *const ecs_iter_t, index: i32) -> *mut T {
	let size = std::mem::size_of::<T>();
	ecs_iter_column_w_size(it, size as size_t, index) as *mut T
}

// This is all WIP!
//
// TODOs:
// - audit & fix up ALL string usages. rust -> C must null terminate!
// - change all get<> component funcs to return Option<>
// - validate that term components were named earlier in chain?
// - We can only safely store primitives and raw pointer types within 
//		components currently, due to how the raw memory is inserted/moved
//		need to look in to hooking the lifecycle support to rust, etc
//		This could become a bit of a deal breaker for idiomatic rust
// 		component storage if not solved

// TODO: make this better. 
// 	possibly we could use world->set_context to hold our custom data container
// 	associated to each world, then inside there cache the comp ids, etc
// 	need to watch for mutable vs readonly worlds
//
// PROBLEM: flecs dupes the world for execution within systems to prevent
// writing to the real world, all mutable operations are deferred. However
// this causes multiple worlds to exist within only the root/real world actually
// having the component ID caches. 
// SOLVED, by the flecs actual world api - ecs_get_world(m_world)
//
// Good resource here:
// https://internals.rust-lang.org/t/generic-type-dependent-static-data/8602

// This might help
// https://docs.rs/generic_static/0.2.0/generic_static/
// Issue with statics however is then we don't get per world comp ids
// and registering is done on the world... we would have to follow the 
// C++ impl and detect that component was already registered prior and
// assume that same ID again...

lazy_static::lazy_static! {
    static ref WORLD_INFOS: Mutex<HashMap<WorldKey, WorldInfoCache>> = {
        let m = HashMap::new();
		Mutex::new(m)
    };
}

type WorldKey = u64;	//*mut ecs_world_t;

#[derive(Copy, Clone, Debug)]
struct ComponentInfo {
	id: u64,
	size: usize,
}

struct WorldInfoCache
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

		// we have to use the actual world in order lookup conponent data
		let actual_world = unsafe { ecs_get_world(world) };
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


pub trait Component : 'static { }
impl<T> Component for T where T: 'static {}



#[cfg(test)]
mod tests {
	use super::*;
	use std::alloc::Layout;

	#[derive(Default, Debug, PartialEq)]
	struct Position {
		x: f32,
		y: f32,
	}

	#[derive(Default, Debug, PartialEq)]
	struct Velocity {
		x: f32,
		y: f32,
	}

	struct Serializable {}

    #[test]
    fn flecs_multiple_worlds() {
		// Component registrations are unique across worlds!
		let mut world1 = World::new();
		let pos1_e = world1.component::<Position>();
		
		let mut world2 = World::new();
		world2.component::<Velocity>();		// insert another comp to steal 1st slot
		let pos2_e = world2.component::<Position>();

		assert_ne!(pos1_e, pos2_e);
	}

    #[test]
    fn flecs_wrappers() {
		let mut world = World::new();
		let pos_e = world.component::<Position>();
		let vel_e = world.component::<Velocity>();
		assert_ne!(pos_e, vel_e);

		let entity = world.entity_builder()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 })
			.build();

		// something broke here??
		let pos = world.get::<Position>(entity).unwrap();
		assert_eq!(pos, &Position { x: 1.0, y: 2.0 });

		let vel = world.get::<Velocity>(entity).unwrap();
		assert_eq!(vel, &Velocity { x: 2.0, y: 4.0 });
	}

    #[test]
    fn flecs_components_are_entities() {
		let mut world = World::new();
		world.component_named::<Position>("Position");	// you can give a comp a name
		world.component::<Serializable>();

		let pos_e = world.id::<Position>().unwrap();
		assert_eq!(world.name(pos_e), "Position");
		
		// It's possible to add components like you would for any entity
		world.add::<Serializable>(pos_e);	
	}

    #[test]
    fn flecs_raw_calls() {
		let world = unsafe { ecs_init() };

		let entity = unsafe { ecs_new_id(world) };
		let is_alive = unsafe { ecs_is_alive(world, entity) };
		assert_eq!(is_alive, true);

		let component = register_component(world, ComponentDescriptor {
			symbol: "flecs::tests::A".to_owned(), 
			name: "A".to_owned(), 
			custom_id: None,
			layout: Layout::from_size_align(16, 4).unwrap()
		});

		let entity = unsafe { ecs_set_id(
			world,
			entity,
			component,
			4,	// size
			b"test".as_ptr() as *const ::std::os::raw::c_void, // ptr
		) };

		// This one should fail/crash due to over size??
		let entity2 = unsafe { ecs_set_id(
			world,
			entity,
			component,
			24,	// size
			b"test12345123451234512345".as_ptr() as *const ::std::os::raw::c_void, // ptr
		) };
		assert_ne!(entity2, 0);

		/*
		// convert this back to readable form...
		let data = unsafe { ecs_get_id(
			world,
			entity,
			component,
		) };	// -> *const ::std::os::raw::c_void;
		assert_eq!(data, b"test".as_ptr() as *const ::std::os::raw::c_void);
		*/

		unsafe { ecs_delete(world, entity) }
		let is_alive = unsafe { ecs_is_alive(world, entity) };
		assert_eq!(is_alive, false);

		unsafe { ecs_fini(world) };
    }
}
