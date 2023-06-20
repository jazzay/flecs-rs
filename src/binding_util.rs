use crate::bindings::*;
use crate::Component;

use once_cell::sync::Lazy;

pub(crate) static NAME_SEP: Lazy<std::ffi::CString> = Lazy::new(|| {
	let sep = std::ffi::CString::new("::").unwrap();
	sep
});

////////////////////////////////////////////////////////////////////////////////////////////////////////
// Impl some flecs Macro like functions that do not bindgen

// This accesses query/filter field component data
pub unsafe fn ecs_field<T: Component>(it: *const ecs_iter_t, index: i32) -> *mut T {
	let size = std::mem::size_of::<T>();
	ecs_field_w_size(it, size, index) as *mut T
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
// String helpers

pub unsafe fn flecs_to_rust_str(cstr: *const ::std::os::raw::c_char) -> &'static str {
	if cstr.is_null() {
		return "";
	}

	// Note we can get strs is coming back with weird numeric encoding
	// which causes the to_str below to fail. Safe guard against that.
	// Update: That was due to components not being registered with a proper name (since Fixed)
	// For now leave this inplace to protect against other bad C strings
	//
	let r_str = std::ffi::CStr::from_ptr(cstr);
	if let Ok(r_str) = r_str.to_str() {
		return r_str;
	}

	// TODO: How should we best handle this?
	"Error"
}

pub unsafe fn flecs_to_rust_string(cstr: *const ::std::os::raw::c_char) -> String {
	if cstr.is_null() {
		return "".into();
	}

	// Note we can get strs is coming back with weird numeric encoding
	// which causes the to_str below to fail. Safe guard against that.
	// Update: That was due to components not being registered with a proper name (since Fixed)
	// For now leave this inplace to protect against other bad C strings
	//
	let r_str = std::ffi::CStr::from_ptr(cstr);
	if let Ok(r_str) = r_str.to_str() {
		return r_str.into();
	}

	// TODO: How should we best handle this?
	"".into()
}
