use super::*;

fn ecs_term<T>(it: *const ecs_iter_t, index: i32) -> *const T {
	let size = std::mem::size_of::<T>();
	unsafe { ecs_term_w_size(it, size as size_t, index) as *const T }
}

pub struct Filter {
	world: *mut ecs_world_t,

	// this has to be on heap due to self-ref fields
	// todo: could look at using Pin or some other stack based strategy
	filter: Box<ecs_filter_t>,	
}

impl Filter {
	pub fn new_2<A: Component, B: Component>(world: *mut ecs_world_t) -> Self {
		let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };

		// TODO: add batch type lookup!
		desc.terms[0].id = WorldInfoCache::component_id_for_type::<A>(world);
		desc.terms[1].id = WorldInfoCache::component_id_for_type::<B>(world);

		let filter: ecs_filter_t = unsafe { MaybeUninit::zeroed().assume_init() };
		let mut filter = Box::new(filter);

		unsafe { ecs_filter_init(world, filter.as_mut(), &desc) };
		// println!("filter: {}, {}, {}", filter.term_cache_used, filter.term_count, filter.term_cache[0].id);
		// println!("filter - terms: {}, {}", filter.terms as u64, filter.term_cache.as_ptr() as u64);
		Filter { world, filter }
	}

	pub fn each<A: Component, B: Component>(&self, mut cb: impl FnMut(&A, &B)) {
		let f = &self.filter;
		// println!("each - filter: {}, {}, {}", f.term_cache_used, f.terms as u64, f.term_cache.as_ptr() as u64);
		unsafe {
			let mut it = ecs_filter_iter(self.world, f.as_ref());
			while ecs_filter_next(&mut it) {
				// Each type has its own set of component arrays
				let a = ecs_term::<A>(&it, 1);
				let b = ecs_term::<B>(&it, 2);
			
				// Iterate all entities for the type
				for i in 0..it.count {
					//printf("%s: {%f, %f}\n", ecs_get_name(world, it.entities[i]), p[i].x, p[i].y);
					let va = a.offset(i as isize);
					let vb = b.offset(i as isize);
					cb(va.as_ref().unwrap(), vb.as_ref().unwrap());
				}
			}
		}		
	}
}

/* Keep related C types handy here for now

/** Filters alllow for ad-hoc quick filtering of entity tables. */
/** Type that describes a single column in the system signature */
typedef struct ecs_term_t {
    ecs_id_t id;                /* Can be used instead of pred, args and role to
                                 * set component/pair id. If not set, it will be 
                                 * computed from predicate, object. If set, the
                                 * subject cannot be set, or be set to This. */
    
    ecs_inout_kind_t inout;     /* Access to contents matched with term */
    ecs_term_id_t pred;         /* Predicate of term */
    ecs_term_id_t args[2];      /* Subject (0), object (1) of term */
    ecs_oper_kind_t oper;       /* Operator of term */
    ecs_id_t role;              /* Role of term */
    char *name;                 /* Name of term */

    int32_t index;              /* Computed term index in filter which takes 
                                 * into account folded OR terms */

    bool move;                  /* When true, this signals to ecs_term_copy that
                                 * the resources held by this term may be moved
                                 * into the destination term. */
} ecs_term_t;

/** Used with ecs_filter_init. */
typedef struct ecs_filter_desc_t {
    /* Terms of the filter. If a filter has more terms than 
     * ECS_TERM_CACHE_SIZE use terms_buffer */
    ecs_term_t terms[ECS_TERM_DESC_CACHE_SIZE];

    /* For filters with lots of terms an outside array can be provided. */
    ecs_term_t *terms_buffer;
    int32_t terms_buffer_count;

    /* Substitute IsA relationships by default. If true, any term with 'set' 
     * assigned to DefaultSet will be modified to Self|SuperSet(IsA). */
    bool substitute_default;

    /* Filter expression. Should not be set at the same time as terms array */
    const char *expr;

    /* Optional name of filter, used for debugging. If a filter is created for
     * a system, the provided name should match the system name. */
    const char *name;
} ecs_filter_desc_t;

struct ecs_filter_t {
    ecs_term_t *terms;         /* Array containing terms for filter */
    int32_t term_count;        /* Number of elements in terms array */
    int32_t term_count_actual; /* Processed count, which folds OR terms */

    ecs_term_t term_cache[ECS_TERM_CACHE_SIZE]; /* Cache for small filters */
    bool term_cache_used;

    bool match_this;           /* Has terms that match EcsThis */
    bool match_only_this;      /* Has only terms that match EcsThis */
    
    char *name;                /* Name of filter (optional) */
    char *expr;                /* Expression of filter (if provided) */

    /* Deprecated fields -- do not use! */
    ecs_type_t include;
    ecs_type_t exclude;
    ecs_match_kind_t include_kind;
    ecs_match_kind_t exclude_kind;
};

*/