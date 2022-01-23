use crate::bindings::*;
use crate::Component;

lazy_static::lazy_static! {
    pub(crate) static ref NAME_SEP: std::ffi::CString = {
		let sep = std::ffi::CString::new("::").unwrap();
		sep
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
// Impl some flecs Macro like functions that do not bindgen

pub(crate) unsafe fn ecs_term_id(it: *const ecs_iter_t, index: i32) -> ecs_id_t {
    assert!(index > 0);		// TODO: later add max check as well
    let index = (index - 1) as usize;
    let term_id = (*it).ids.add(index);
    *term_id
}

pub(crate) unsafe fn ecs_term_source(it: *const ecs_iter_t, index: i32) -> ecs_entity_t {
    assert!(index > 0);		// TODO: later add max check as well
    if (*it).subjects.is_null() {
        0
    } else {
        let index = (index - 1) as usize;
        *((*it).subjects.add(index))
    } 
}

pub(crate) unsafe fn ecs_term_size(it: *const ecs_iter_t, index: i32) -> size_t {
    assert!(index > 0);		// TODO: later add max check as well
    *((*it).sizes.add((index - 1) as usize)) as size_t
}

pub(crate) unsafe fn ecs_term_is_owned(it: *const ecs_iter_t, index: i32) -> bool {
    assert!(index > 0);		// TODO: later add max check as well
    let index = (index - 1) as usize;
    (*it).subjects.is_null() || *((*it).subjects.add(index)) == 0
}

// This access query/filter term component data
pub(crate) unsafe fn ecs_term<T: Component>(it: *const ecs_iter_t, index: i32) -> *mut T {
    let size = std::mem::size_of::<T>();
    ecs_term_w_size(it, size as size_t, index) as *mut T
}

// This accesses all table columns for a matched archetype
pub(crate) unsafe fn ecs_iter_column<T: Component>(it: *const ecs_iter_t, index: i32) -> *mut T {
    let size = std::mem::size_of::<T>();
    ecs_iter_column_w_size(it, size as size_t, index) as *mut T
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
// Vector / Sring helpers

pub(crate) unsafe fn ecs_vector_first<T: Sized>(vector: *const ecs_vector_t) -> *const T {
    // TODO: Should pull this out in to helpers like above
    let vector_size = std::mem::size_of::<ecs_vector_t>() as i16;
    let elem_size = std::mem::size_of::<T>() as i32;
    let elem_align = std::mem::align_of::<T>() as i16;
    let offset = vector_size.max(elem_align);

    let first = _ecs_vector_first(vector, elem_size, offset) as *const T;
    first
}

pub(crate) unsafe fn flecs_to_rust_str(cstr: *const ::std::os::raw::c_char) -> &'static str {
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