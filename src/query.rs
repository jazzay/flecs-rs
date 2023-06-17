use crate::*;

pub struct Query {
	world: *mut ecs_world_t,
	query: *mut ecs_query_t,	
}

impl Query {
	// TODO - performance is poor in these each methods compared to iter (by 20x) 
	// due to looking up terms for each tuple entry * each entity. We need to rework this 
	// to take slices of components, determined outside the loop so optimal iteration can occur
	//
	pub fn each<'w, G: ComponentGroup<'w>>(&'w self, mut cb: impl FnMut(Entity, G::RefTuple)) {
		unsafe {
			let mut it = ecs_query_iter(self.world, self.query);
			while ecs_query_next(&mut it) {
				// Iterate all entities for the type
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(self.world, *eid);
					let rt = G::iter_as_ref_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		}				
	}

	pub fn each_mut<'w, G: ComponentGroup<'w>>(&mut self, mut cb: impl FnMut(Entity, G::MutRefTuple)) {
		unsafe {
			let mut it = ecs_query_iter(self.world, self.query);
			while ecs_query_next(&mut it) {
				// Iterate all entities for the type
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(self.world, *eid);
					let rt = G::iter_as_mut_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		}				
	}

	pub fn iter<F: FnMut(&Iter)>(&self, mut func: F) {
		unsafe {
			let mut it = ecs_query_iter(self.world, self.query);
			while ecs_query_next(&mut it) {
				let iter = Iter::new(&mut it);
				func(&iter);
			}
		}				
	}
}

pub struct QueryBuilder<'w> {
	world: &'w World,
	desc: ecs_query_desc_t,
	next_term_index: usize,
}

impl<'w> TermBuilder for QueryBuilder<'w> {
    fn world(&mut self) -> *mut ecs_world_t {
        self.world.raw()
    }

	fn filter_desc(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc.filter
	}

    fn current_term(&mut self) -> &mut ecs_term_t {
        &mut self.desc.filter.terms[self.next_term_index]
    }

    fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}

impl<'w> QueryBuilder<'w> {
	pub fn new(world: &'w World) -> Self {
		Self { 
			world,
			desc: unsafe { MaybeUninit::zeroed().assume_init() },
			next_term_index: 0
		}
	}

	pub fn build(self) -> Query {
		let query = unsafe { ecs_query_init(self.world.raw(), &self.desc) };
		Query { 
			world: self.world.raw(), 
			query 
		}
	}
}
