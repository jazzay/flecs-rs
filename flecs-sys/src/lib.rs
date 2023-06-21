#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Allow some bindgen warnings for now
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

use std::mem::MaybeUninit;

pub mod bindings;
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
