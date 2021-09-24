use crate::*;

// This is all WIP!

pub fn register_component(world: *mut ecs_world_t, name: &str, layout: std::alloc::Layout) -> ecs_entity_t {
	// How C code registers a component
	//ECS_COMPONENT(world, Position);
	// expands into:

    // ecs_id_t ecs_id(id) = ecs_component_init(world, &(ecs_component_desc_t){\
    //     .entity = {\
    //         .name = #id,\
    //         .symbol = #id\
    //     },\
    //     .size = sizeof(id),\
    //     .alignment = ECS_ALIGNOF(id)\
    // });\
    // ECS_VECTOR_STACK(FLECS__T##id, ecs_entity_t, &FLECS__E##id, 1);\
    // (void)ecs_id(id);\
    // (void)ecs_type(id)	

	let mut e_desc: ecs_entity_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
	e_desc.name = name.as_ptr() as *const i8;

	// let entity_desc = ecs_entity_desc_t {
	// 	name: "Hello".as_ptr() as *const c_char,
		// pub entity: ecs_entity_t,
		// pub name: *const ::std::os::raw::c_char,
		// pub sep: *const ::std::os::raw::c_char,
		// pub root_sep: *const ::std::os::raw::c_char,
		// pub symbol: *const ::std::os::raw::c_char,
		// pub use_low_id: bool,
		// pub add: [ecs_id_t; 32usize],
		// pub remove: [ecs_id_t; 32usize],
		// pub add_expr: *const ::std::os::raw::c_char,
		// pub remove_expr: *const ::std::os::raw::c_char,
	// }
	
	let comp_desc = ecs_component_desc_t {
		entity: e_desc,
		size: layout.size() as u64,
		alignment: layout.align() as u64,
	};

	let comp_entity = unsafe { ecs_component_init(world, &comp_desc) };
	comp_entity
}

fn pod_component<T>(
    world: *mut ecs_world_t, 
    name: Option<&str>, 
    allow_tag: bool, 
	id: Option<ecs_id_t>) -> Option<Entity>
{
/*	
    const char *n = name;
    bool implicit_name = false;
    if (!n) {
        n = _::name_helper<T>::name();

        /* Keep track of whether name was explicitly set. If not, and the 
         * component was already registered, just use the registered name.
         *
         * The registered name may differ from the typename as the registered
         * name includes the flecs scope. This can in theory be different from
         * the C++ namespace though it is good practice to keep them the same */
        implicit_name = true;
    }

    if (_::cpp_type<T>::registered()) {
        /* Obtain component id. Because the component is already registered,
         * this operation does nothing besides returning the existing id */
        id = _::cpp_type<T>::id_explicit(world, name, allow_tag, id);

        /* If entity has a name check if it matches */
        if (ecs_get_name(world, id) != nullptr) {
            if (!implicit_name && id >= EcsFirstUserComponentId) {
                char *path = ecs_get_path_w_sep(
                    world, 0, id, "::", nullptr);
                ecs_assert(!strcmp(path, n), 
                    ECS_INCONSISTENT_NAME, name);
                ecs_os_free(path);
            }
        } else {
            /* Register name with entity, so that when the entity is created the
             * correct id will be resolved from the name. Only do this when the
             * entity is empty.*/
            ecs_add_path_w_sep(world, id, 0, n, "::", "::");
        }

        /* If a component was already registered with this id but with a 
         * different size, the ecs_component_init function will fail. */

        /* We need to explicitly call ecs_component_init here again. Even though
         * the component was already registered, it may have been registered
         * with a different world. This ensures that the component is registered
         * with the same id for the current world. 
         * If the component was registered already, nothing will change. */
        ecs_component_desc_t desc = {};
        desc.entity.entity = id;
        desc.size = _::cpp_type<T>::size();
        desc.alignment = _::cpp_type<T>::alignment();
        ecs_entity_t entity = ecs_component_init(world, &desc);
        (void)entity;
        
        ecs_assert(entity == id, ECS_INTERNAL_ERROR, NULL);

        /* This functionality could have been put in id_explicit, but since
         * this code happens when a component is registered, and the entire API
         * calls id_explicit, this would add a lot of overhead to each call.
         * This is why when using multiple worlds, components should be 
         * registered explicitly. */
    } else {
        /* If the component is not yet registered, ensure no other component
         * or entity has been registered with this name. Ensure component is 
         * looked up from root. */
        ecs_entity_t prev_scope = ecs_set_scope(world, 0);
        ecs_entity_t entity;
        if (id) {
            entity = id;
        } else {
            entity = ecs_lookup_path_w_sep(world, 0, n, "::", "::", false);
        }

        ecs_set_scope(world, prev_scope);

        /* If entity exists, compare symbol name to ensure that the component
         * we are trying to register under this name is the same */
        if (entity) {
            if (!id) {
                const char *sym = ecs_get_symbol(world, entity);
                ecs_assert(sym != NULL, ECS_INTERNAL_ERROR, NULL);
                (void)sym;

                char *symbol = _::symbol_helper<T>::symbol();
                ecs_assert(!ecs_os_strcmp(sym, symbol), ECS_NAME_IN_USE, n);
                ecs_os_free(symbol);

            /* If an existing id was provided, it's possible that this id was
             * registered with another type. Make sure that in this case at
             * least the component size/alignment matches.
             * This allows applications to alias two different types to the same
             * id, which enables things like redefining a C type in C++ by
             * inheriting from it & adding utility functions etc. */
            } else {
                const EcsComponent *comp = ecs_get(world, entity, EcsComponent);
                if (comp) {
                    ecs_assert(comp->size == ECS_SIZEOF(T),
                        ECS_INVALID_COMPONENT_SIZE, NULL);
                    ecs_assert(comp->alignment == ECS_ALIGNOF(T),
                        ECS_INVALID_COMPONENT_ALIGNMENT, NULL);
                } else {
                    /* If the existing id is not a component, no checking is
                     * needed. */
                }
            }

        /* If no entity is found, lookup symbol to check if the component was
         * registered under a different name. */
        } else {
            char *symbol = _::symbol_helper<T>::symbol();
            entity = ecs_lookup_symbol(world, symbol, false);
            ecs_assert(entity == 0, ECS_INCONSISTENT_COMPONENT_ID, symbol);
            ecs_os_free(symbol);
        }

        /* Register id as usual */
        id = _::cpp_type<T>::id_explicit(world, name, allow_tag, id);
    }
*/    
    return None;	//flecs::entity(world, id);
}