#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

use std::{any::TypeId, mem::MaybeUninit};

// We generate bindings to an actual source file so that we get better IDE integration

// For now do not export Docs for all the Raw bindings.
// Sadly to publish on crates.io we cannot write outside the OUT_DIR
// revisit this later.
// We will need to expose types that are part of the Rust api at some point
// #[doc(hidden)]
// mod bindings;
// #[doc(hidden)]
// pub use bindings::*;

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use bindings::*;

mod binding_util;
pub use binding_util::*;

mod cache; // Internal only

mod component;
pub use component::*;

mod component_group;
pub use component_group::*;

mod entity;
pub use entity::*;

pub mod filter;
pub use filter::*;

pub mod id;
pub use id::*;

pub mod query;
pub use query::*;

pub mod system;
pub use system::*;

pub mod terms;
pub use terms::*;

pub mod world;
pub use world::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////
// This Rust binding for flecs is a WIP!!!
//
// Possible TODOs:
// - audit & fix up ALL string usages. rust -> C must null terminate!
// - change all get<> component funcs to return Option<>?
// - validate that term components were named earlier in chain?
// - We can only safely store primitives and raw pointer types within
//		components currently, due to how the raw memory is inserted/moved
//		need to look in to hooking the lifecycle support to rust, etc
//		This could become a bit of a deal breaker for idiomatic rust
// 		component storage if not solved
// - Implement proper Rusty Query / System APIs that use Tuple generics

pub trait Component: 'static {}
impl<T> Component for T where T: 'static {}

pub trait AsEcsId {
    fn id(&self) -> ecs_id_t;
}

impl AsEcsId for EntityId {
    fn id(&self) -> ecs_id_t {
        *self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
// C Struct initializer Defaults
//
impl Default for ecs_entity_desc_t {
    fn default() -> Self {
        let desc: ecs_entity_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
        desc
    }
}

impl Default for ecs_system_desc_t {
    fn default() -> Self {
        let desc: ecs_system_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
        desc
    }
}

// TODO - port more C++ tests to Rust!!!
//
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
        world2.component::<Velocity>(); // insert another comp to steal 1st slot
        let pos2_e = world2.component::<Position>();

        assert_ne!(pos1_e, pos2_e);
    }

    #[test]
    fn flecs_wrappers() {
        let mut world = World::new();
        let pos_e = world.component::<Position>();
        let vel_e = world.component::<Velocity>();
        assert_ne!(pos_e, vel_e);

        let entity = world
            .entity()
            .set(Position { x: 1.0, y: 2.0 })
            .set(Velocity { x: 2.0, y: 4.0 });

        // something broke here??
        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos, &Position { x: 1.0, y: 2.0 });

        let vel = world.get::<Velocity>(entity).unwrap();
        assert_eq!(vel, &Velocity { x: 2.0, y: 4.0 });
    }

    #[test]
    fn flecs_components_are_entities() {
        let mut world = World::new();
        world.component_named::<Position>("Position"); // you can give a comp a name
        world.component::<Serializable>();

        let pos_e = world.id::<Position>().unwrap();
        assert_eq!(world.name(pos_e), "Position");

        // It's possible to add components like you would for any entity
        world.add::<Serializable>(pos_e);
    }

    #[test]
    fn flecs_raw_binding_calls() {
        let world = unsafe { ecs_init() };

        let entity = unsafe { ecs_new_id(world) };
        let is_alive = unsafe { ecs_is_alive(world, entity) };
        assert_eq!(is_alive, true);

        let component = register_component(
            world,
            ComponentDescriptor {
                symbol: "flecs::tests::A".to_owned(),
                name: "A".to_owned(),
                custom_id: None,
                layout: Layout::from_size_align(16, 4).unwrap(),
            },
        );

        let entity = unsafe {
            ecs_set_id(
                world,
                entity,
                component,
                4,                                                 // size
                b"test".as_ptr() as *const ::std::os::raw::c_void, // ptr
            )
        };

        // This one should fail/crash due to over size??
        let entity2 = unsafe {
            ecs_set_id(
                world,
                entity,
                component,
                24,                                                                    // size
                b"test12345123451234512345".as_ptr() as *const ::std::os::raw::c_void, // ptr
            )
        };
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
