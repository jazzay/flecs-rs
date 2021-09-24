#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

// use std::os::raw::c_char;
use std::{any::TypeId, collections::HashMap, mem::{MaybeUninit}, sync::Mutex};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod component;
use component::*;

// This is all WIP!

lazy_static::lazy_static! {
    static ref TYPE_MAP: Mutex<HashMap<TypeId, u64>> = {
        let m = HashMap::new();
		Mutex::new(m)
    };
}

pub trait Component : 'static { }
impl<T> Component for T where T: 'static {}

pub fn component_id_for_type<T: Component>() -> ecs_entity_t {
	// component MUST be registered ahead of time!
	let type_id = TypeId::of::<T>();
	let comp_id = TYPE_MAP.lock().unwrap().get(&type_id).unwrap().clone();	
	comp_id
}

pub struct Entity {
	entity: ecs_entity_t,
	world: *mut ecs_world_t,
}

impl Entity {
	pub fn new(entity: ecs_entity_t, world: *mut ecs_world_t) -> Self {
		Self { entity, world }
	}

	pub fn get<T: Component>(&self) -> &T {
		let comp_id = component_id_for_type::<T>();
		let value = unsafe { ecs_get_id(self.world, self.entity, comp_id) };
		unsafe { (value as *const T).as_ref().unwrap() }
	}

    pub fn get_mut<T: Component>(&mut self/*bool *is_added = nullptr*/) -> &mut T  {	
		let comp_id = component_id_for_type::<T>();
		let mut is_added = false;
		let value = unsafe { ecs_get_mut_w_entity(self.world, self.entity, comp_id, &mut is_added) };
		unsafe { (value as *mut T).as_mut().unwrap() }
    }

	pub fn set<T: Component>(&mut self, value: T) {
		let dest = self.get_mut::<T>();
		*dest = value;
	}
}

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

	pub fn entity_new(&mut self) -> Entity {
		let entity = unsafe { ecs_new_id(self.world) };
		Entity::new(entity, self.world)
	}
	
    pub fn progress(&self, delta_time: f32) -> bool {
        return unsafe { ecs_progress(self.world, delta_time) }
    }	

	fn lookup(name: &str) -> Option<Entity> {
		None
	}

	fn component<T: 'static>(&mut self) -> Entity {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = TYPE_MAP.lock().unwrap().get(&type_id) {
			return Entity::new(*comp_id, self.world);
		}

		// let result: Entity = pod_component<T>(world, name);
	
		// if (_::cpp_type<T>::size()) {
		// 	_::register_lifecycle_actions<T>(world, result);
		// }
		
		let type_name = std::any::type_name::<T>();
		let layout = std::alloc::Layout::new::<T>();
		let comp_id = register_component(self.world, type_name, layout);
		TYPE_MAP.lock().unwrap().insert(type_id, comp_id);
		Entity::new(comp_id, self.world)
	}	
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::alloc::Layout;

	#[derive(Default, Debug, PartialEq)]
	struct Position {
		x: f32,
		y: f32,
	}

    #[test]
    fn flecs_wrappers() {
		let mut world = World::new();
		let posC = world.component::<Position>();

		let mut entity = world.entity_new();
		entity.set(Position { x: 1.0, y: 2.0 });

		let pos = entity.get::<Position>();
		assert_eq!(pos, &Position { x: 1.0, y: 2.0 });

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
