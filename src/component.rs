use std::alloc::Layout;

use crate::*;
use crate::cache::WorldInfoCache;

// This is WIP!

// Notes:
// I could leverage flecs aliasing to give engine/internal components public names.
//		for example Position {} -> 'module.Position'
//		then plugins could lookup/cache the runtime id via those names

pub(crate) fn register_component_typed<T: 'static>(world: *mut ecs_world_t, name: Option<&str>) -> EntityId {
	// see if we already cached it
	if let Some(comp_id) = WorldInfoCache::get_component_id_for_type::<T>(world) {
		return comp_id;
	}

	let type_id = TypeId::of::<T>();
	let layout = std::alloc::Layout::new::<T>();
	let symbol = std::any::type_name::<T>().to_owned();

	// Need to figure out best way to 'Auto-Name' components based on the rust type name.
	// By default we would want the struct name only so that queries, etc match
	//
	let name = if let Some(name) = name {
		name.to_owned()
	} else {
		// Note :: in rust is the module sep, while in flecs it is path sep (parenting)
		let s = symbol.replace("::", ".");
		s.split(".").last().unwrap().to_owned()
	};

	// To achieve language neutral component symbol/naming we need to strip off any compiler
	// specific aspects of the symbol as well. But this may not jive with general Flecs-rs users...
	let symbol = name.clone();

	let comp_id = register_component(world, 
		ComponentDescriptor { 
			symbol,
			name, 
			custom_id: None,
			layout 
	});

    //println!("Registered Component: {} -> {}", symbol, comp_id);
	WorldInfoCache::register_component_id_for_type_id(world, comp_id, type_id);
	comp_id
}

pub(crate) fn register_component_dynamic(world: *mut ecs_world_t, symbol: &'static str, name: Option<&'static str>, layout: Layout) -> EntityId {
	// see if we already cached it
	if let Some(comp_info) = WorldInfoCache::get_component_id_for_symbol(world, symbol) {
		return comp_info.id;
	}
	let comp_id = register_component(world, 
		ComponentDescriptor { 
			symbol: symbol.to_owned(), 
			name: name.unwrap_or("").to_owned(), 
			custom_id: None,
			layout 
	});

	WorldInfoCache::register_component_id_for_symbol(world, comp_id, symbol, layout.size());
	comp_id
}

// Looks up the EcsComponent data on a Component entity
pub(crate) fn get_component_info(world: *mut ecs_world_t, comp_e: ecs_entity_t) -> Option<EcsComponent> {
	// flecs stores info about components (size, align) within the world
	// these are built-in components which we can acess via special component ids
	let id = unsafe { FLECS__EEcsComponent as u64 };
	let raw = unsafe { ecs_get_id(world, comp_e, id) };	
	if raw.is_null() {
		return None;
	}

	let c = unsafe { (raw as *const EcsComponent).as_ref().unwrap() };
	// println!("Got Component info for: {}, size: {}, align: {}", comp_e, c.size, c.alignment);
	Some(c.clone())
}

#[derive(Debug)]
pub struct ComponentDescriptor {
	pub symbol: String, 
	pub name: String, 
	pub custom_id: Option<u64>,
	pub layout: std::alloc::Layout
}

pub fn register_component(world: *mut ecs_world_t, desc: ComponentDescriptor) -> ecs_entity_t {
	// println!("register_component - {:?}", desc);

	let name_c_str = std::ffi::CString::new(desc.name).unwrap();
	let symbol_c_str = std::ffi::CString::new(desc.symbol).unwrap();

	// could be a const
	let sep = std::ffi::CString::new("::").unwrap();

	let mut entity_desc: ecs_entity_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
	if let Some(custom_id) = desc.custom_id {
		entity_desc.id = custom_id;
	}

	// For now these are the same as the T::name is passed in
	entity_desc.name = name_c_str.as_ptr() as *const i8;
	entity_desc.symbol = symbol_c_str.as_ptr() as *const i8;

	entity_desc.sep = sep.as_ptr() as *const i8;
	entity_desc.root_sep = sep.as_ptr() as *const i8;

    let entity = unsafe { ecs_entity_init(world, &entity_desc) };

	// only register a ecs component if size > 0
	if desc.layout.size() > 0 {
		let mut comp_desc: ecs_component_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
		comp_desc.entity = entity;
		comp_desc.type_.size = desc.layout.size() as ecs_size_t;
		comp_desc.type_.alignment = desc.layout.align() as ecs_size_t;

		let comp_entity = unsafe { ecs_component_init(world, &comp_desc) };
		assert!(comp_entity == entity);
	}

	// println!("register_component - entity {}", entity);

	entity
}
