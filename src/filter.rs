use crate::*;
use crate::cache::WorldInfoCache;

// TODO - This will be merged with FilterGroup once we solve Single elem tuples
//
pub struct Filter {
	world: *mut ecs_world_t,

	// this has to be on heap due to self-ref fields
	// todo: could look at using Pin or some other stack based strategy
	filter: Box<ecs_filter_t>,	
}

// TODO - need to support generalized API via tuples or something
impl Filter {
	pub fn new_1<A: Component>(world: *mut ecs_world_t) -> Self {
		let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };

		// TODO: add batch type lookup!
		desc.terms[0].id = WorldInfoCache::get_component_id_for_type::<A>(world).expect("Component type not registered!");

		let filter: ecs_filter_t = unsafe { MaybeUninit::zeroed().assume_init() };
		let mut filter = Box::new(filter);

		unsafe { ecs_filter_init(world, filter.as_mut(), &desc) };
		Filter { world, filter }
	}

	pub fn each_1<A: Component>(&self, mut cb: impl FnMut(Entity, &A)) {
		let f = &self.filter;
		unsafe {
			let mut it = ecs_filter_iter(self.world, f.as_ref());
			while ecs_filter_next(&mut it) {
				let a = ecs_term::<A>(&it, 1);
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(self.world, *eid);
					let va = a.offset(i as isize);
					cb(e, va.as_ref().unwrap());
				}
			}
		}		
	}

}


pub struct FilterGroup<'c, G: ComponentGroup<'c>> {
	world: &'c World,

	// this has to be on heap due to self-ref fields
	// todo: could look at using Pin or some other stack based strategy
	filter: Box<ecs_filter_t>,	

	_phantom: std::marker::PhantomData<G>,
}

// TODO - need to support generalized API via tuples or something
impl<'c, G: ComponentGroup<'c>> FilterGroup<'c, G> {
	pub fn new(world: &'c World) -> Self {
		let world_raw = world.raw();
		let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
		unsafe { G::fill_descriptor(world_raw, &mut desc) };

		let filter: ecs_filter_t = unsafe { MaybeUninit::zeroed().assume_init() };
		let mut filter = Box::new(filter);

		unsafe { ecs_filter_init(world_raw, filter.as_mut(), &desc) };
		FilterGroup { 
			world, 
			filter,
			_phantom: Default::default(),
		}
	}

	pub fn each(&self, mut cb: impl FnMut(Entity, G::RefTuple)) {
		let world_raw = self.world.raw();
		let f = &self.filter;
		// println!("each - filter: {}, {}, {}", f.term_cache_used, f.terms as u64, f.term_cache.as_ptr() as u64);
		unsafe {
			let mut it = ecs_filter_iter(world_raw, f.as_ref());
			while ecs_filter_next(&mut it) {
				// Iterate all entities for the type
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(world_raw, *eid);
					let rt = G::iter_as_ref_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		}				
	}

	pub fn each_mut(&self, mut cb: impl FnMut(Entity, G::MutRefTuple)) {
		let world_raw = self.world.raw();
		let f = &self.filter;
		// println!("each_mut - filter: {}, {}, {}", f.term_cache_used, f.terms as u64, f.term_cache.as_ptr() as u64);
		unsafe {
			let mut it = ecs_filter_iter(world_raw, f.as_ref());
			while ecs_filter_next(&mut it) {
				// Iterate all entities for the type
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(world_raw, *eid);
					let rt = G::iter_as_mut_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		}				
	}
}
