#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

use std::mem::MaybeUninit;

// We generate bindings to an actual source file so that we get better IDE integration

// For now do not export Docs for all the Raw bindings.
// Sadly to publish on crates.io we cannot write outside the OUT_DIR
// revisit this later.
// We will need to expose types that are part of the Rust api at some point
// #[doc(hidden)]
// mod bindings;
// #[doc(hidden)]
// pub use bindings::*;

pub mod bindings {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

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
