#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

// use std::os::raw::c_char;
use std::mem::{MaybeUninit};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod component;
use component::*;

// This is all WIP!

pub struct Entity {
	entity: ecs_entity_t,
}

impl Entity {
	pub fn new(entity: ecs_entity_t) -> Self {
		Self { entity }
	}

	pub fn get<T: Default>(&mut self) -> T {
		T::default()
	}

    pub fn get_mut<T>(&mut self/*bool *is_added = nullptr*/) -> *const T  {		
        // auto comp_id = _::cpp_type<T>::id(m_world);
        // ecs_assert(_::cpp_type<T>::size() != 0, ECS_INVALID_PARAMETER, NULL);
        // return static_cast<T*>(
        //     ecs_get_mut_w_entity(m_world, m_id, comp_id, is_added));
		std::ptr::null()
    }

	pub fn set<T>(&mut self, value: T) {
	}
}

pub struct World {
	world: *mut ecs_world_t	
}

impl World {
	pub fn new() -> Self {
		let world = unsafe { ecs_init() };
		//init_builtin_components();
		Self {
			world
		}
	}

	pub fn entity_new(&self) -> Entity {
		let entity = unsafe { ecs_new_id(self.world) };
		Entity { entity }
	}
	
    pub fn progress(&self, delta_time: f32) -> bool {
        return unsafe { ecs_progress(self.world, delta_time) }
    }	

	fn lookup(name: &str) -> Option<Entity> {
		None
	}

	fn component<T>(&self, name: &str) -> Entity {
		// let result: Entity = pod_component<T>(world, name);
	
		// if (_::cpp_type<T>::size()) {
		// 	_::register_lifecycle_actions<T>(world, result);
		// }
		
		let type_name = std::any::type_name::<T>();
		let layout = std::alloc::Layout::new::<T>();
		let comp = register_component(self.world, type_name, layout);
		Entity::new(comp)
	}	
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::alloc::Layout;

	#[derive(Default, Debug)]
	struct Position {
		x: f32,
		y: f32,
	}

    #[test]
    fn flecs_wrappers() {
		let world = World::new();
		let posC = world.component::<Position>("Position");

		let mut entity = world.entity_new();
		entity.set(Position { x: 0.0, y: 1.0 });
		let pos = entity.get::<Position>();
		println!("Pos = {:?}", pos);
	}

    #[test]
    fn flecs_raw_calls() {
		let world = unsafe { ecs_init() };

		let entity = unsafe { ecs_new_id(world) };
		let is_alive = unsafe { ecs_is_alive(world, entity) };
		assert_eq!(is_alive, true);

		let component = register_component(world, "test", Layout::from_size_align(16, 4).unwrap());

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
