#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

use std::{any::TypeId, collections::HashMap, mem::{MaybeUninit}, sync::Mutex};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod component;
use component::*;

mod entity;
use entity::*;

pub mod filter;
pub use filter::*;

pub mod world;
pub use world::*;

// This is all WIP!
//
// TODOs:
// - fix up string usage. rust -> C must null terminate!

// This is causing problems in tests, as new worlds are created
// but this does not get cleared. need a better strategy.
// for now just reset it at end of test (when world is dropped?)
lazy_static::lazy_static! {
    static ref WORLD_INFOS: Mutex<HashMap<WorldKey, WorldInfoCache>> = {
        let m = HashMap::new();
		Mutex::new(m)
    };
}

type WorldKey = u64;	//*mut ecs_world_t;

struct WorldInfoCache
{
	component_typeid_map: HashMap<TypeId, u64>,
}

impl WorldInfoCache {
	pub(crate) fn insert(world: *mut ecs_world_t) {
		let cache = WorldInfoCache {
			component_typeid_map: HashMap::new()
		};

		let world_key = Self::key_for_world(world);
		let mut m = WORLD_INFOS.lock().unwrap();
		m.insert(world_key, cache);
	}

	fn key_for_world(world: *mut ecs_world_t) -> u64 {
		world as u64
	}

	pub fn component_id_for_type<T: Component>(world: *mut ecs_world_t) -> ecs_entity_t {
		let world_key = Self::key_for_world(world);
		let m = WORLD_INFOS.lock().unwrap();
		let cache = m.get(&world_key).unwrap();	//.clone();	

		// component MUST be registered ahead of time!
		let type_id = TypeId::of::<T>();
		let comp_id = cache.component_typeid_map.get(&type_id).unwrap().clone();	
		comp_id
	}

	pub fn try_get_component_id_for_type<T: Component>(world: *mut ecs_world_t) -> Option<ecs_entity_t> {
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
		let pos1_e = world1.component::<Position>(None);
		
		let mut world2 = World::new();
		world2.component::<Velocity>(None);		// insert another comp to steal 1st slot
		let pos2_e = world2.component::<Position>(None);

		assert_ne!(pos1_e, pos2_e);
	}

    #[test]
    fn flecs_wrappers() {
		let mut world = World::new();
		let pos_e = world.component::<Position>(None);
		let vel_e = world.component::<Velocity>(None);
		assert_ne!(pos_e, vel_e);

		let entity = world.entity_builder()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 })
			.build();

		// something broke here??
		let pos = world.get::<Position>(entity);
		assert_eq!(pos, &Position { x: 1.0, y: 2.0 });

		let vel = world.get::<Velocity>(entity);
		assert_eq!(vel, &Velocity { x: 2.0, y: 4.0 });
	}

    #[test]
    fn flecs_components_are_entities() {
		let mut world = World::new();
		world.component::<Position>(Some("Position"));	// you can give a comp a name
		world.component::<Serializable>(None);

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

		let component = register_component(world, Some("A"), "flecs::tests::A", Layout::from_size_align(16, 4).unwrap());

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
