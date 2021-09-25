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

pub mod filter;
pub use filter::*;

// This is all WIP!
//
// TODOs:
// - fix up string usage. rust -> C must null terminate!

// This is causing problems in tests, as new worlds are created
// but this does not get cleared. need a better strategy.
// for now just reset it at end of test (when world is dropped?)
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

#[derive(PartialEq, Eq, Debug)]
pub struct Entity {
	entity: ecs_entity_t,
	world: *mut ecs_world_t,
}

impl Entity {
	pub fn new(entity: ecs_entity_t, world: *mut ecs_world_t) -> Self {
		Self { entity, world }
	}

	pub fn name(&self) -> &str {
		let char_ptr = unsafe { ecs_get_name(self.world, self.entity) };
		let c_str = unsafe { std::ffi::CStr::from_ptr(char_ptr) };
		let name = c_str.to_str().unwrap();
		println!("name(): {}", name);
		name
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

	pub fn set<T: Component>(&mut self, value: T) -> &mut Self {
		let dest = self.get_mut::<T>();
		*dest = value;
		self
	}

	pub fn add<T: Component>(&mut self) -> &mut Self {
        // flecs_static_assert(is_flecs_constructible<T>::value,
        //     "cannot default construct type: add T::T() or use emplace<T>()");
		let comp_id = component_id_for_type::<T>();
        unsafe { ecs_add_id(self.world, self.entity, comp_id) };
		self
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

	pub fn raw(&self) -> *mut ecs_world_t {
		self.world
	}

	pub fn entity(&mut self) -> Entity {
		let entity = unsafe { ecs_new_id(self.world) };
		Entity::new(entity, self.world)
	}
	
    pub fn progress(&self, delta_time: f32) -> bool {
        return unsafe { ecs_progress(self.world, delta_time) }
    }	

	pub fn lookup(name: &str) -> Option<Entity> {
		None
	}

	pub fn id<T: Component>(&mut self) -> Option<Entity> {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = TYPE_MAP.lock().unwrap().get(&type_id) {
			return Some(Entity::new(*comp_id, self.world));
		}
		None
	}

	pub fn component<T: 'static>(&mut self, name: Option<&str>) -> Entity {
		let type_id = TypeId::of::<T>();

		// see if we already cached it
		if let Some(comp_id) = TYPE_MAP.lock().unwrap().get(&type_id) {
			return Entity::new(*comp_id, self.world);
		}

		// let result: Entity = pod_component<T>(world, name);
	
		// if (_::cpp_type<T>::size()) {
		// 	_::register_lifecycle_actions<T>(world, result);
		// }
		
		let symbol = std::any::type_name::<T>();
		let layout = std::alloc::Layout::new::<T>();
		let comp_id = register_component(self.world, name, symbol, layout);
		TYPE_MAP.lock().unwrap().insert(type_id, comp_id);
		Entity::new(comp_id, self.world)
	}	
}

impl Drop for World {
	fn drop(&mut self) {
		TYPE_MAP.lock().unwrap().clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::filter::*;
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
    fn flecs_wrappers() {
		let mut world = World::new();
		let pos_e = world.component::<Position>(None);
		let vel_e = world.component::<Velocity>(None);
		assert_ne!(pos_e, vel_e);

		let mut entity = world.entity();
		entity.set(Position { x: 1.0, y: 2.0 });
		entity.set(Velocity { x: 2.0, y: 4.0 });

		// something broke here??
		let pos = entity.get::<Position>();
		assert_eq!(pos, &Position { x: 1.0, y: 2.0 });

		let vel = entity.get::<Velocity>();
		assert_eq!(vel, &Velocity { x: 2.0, y: 4.0 });
	}

    #[test]
    fn flecs_components_are_entities() {
		let mut world = World::new();
		world.component::<Position>(Some("Position"));	// you can give a comp a name
		world.component::<Serializable>(None);

		let mut pos_e = world.id::<Position>().unwrap();
		assert_eq!(pos_e.name(), "Position");
		
		// It's possible to add components like you would for any entity
		pos_e.add::<Serializable>();	
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
