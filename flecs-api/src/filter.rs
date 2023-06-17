use std::mem::MaybeUninit;

use crate::*;
use crate::cache::WorldInfoCache;

// TODO - This will be merged with FilterGroup once we solve Single elem tuples
//
pub struct Filter {
	world: *mut ecs_world_t,
	filter: *mut ecs_filter_t,	
}

// TODO - need to support generalized API via tuples or something
impl Filter {
	pub fn new_1<A: Component>(world: *mut ecs_world_t) -> Self {
		let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };

		// TODO: add batch type lookup!
		desc.terms[0].id = WorldInfoCache::get_component_id_for_type::<A>(world).expect("Component type not registered!");

		let filter = unsafe { ecs_filter_init(world, &desc) };
		Filter { world, filter }
	}

	pub fn each_1<A: Component>(&self, mut cb: impl FnMut(Entity, &A)) {
		unsafe {
			let mut it = ecs_filter_iter(self.world, self.filter);
			while ecs_filter_next(&mut it) {
				let a = ecs_field::<A>(&it, 1);
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(self.world, *eid);
					let va = a.offset(i as isize);
					cb(e, va.as_ref().unwrap());
				}
			}
		}		
	}

	pub fn each<'w, G: ComponentGroup<'w>>(&'w self, mut cb: impl FnMut(Entity, G::RefTuple)) {
		unsafe {
			let mut it = ecs_filter_iter(self.world, self.filter);
			while ecs_filter_next(&mut it) {
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
			let mut it = ecs_filter_iter(self.world, self.filter);
			while ecs_filter_next(&mut it) {
				// Iterate all entities for the type
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(self.world, *eid);

					// TODO - performance is poor here due to looking up terms for each tuple entry * each entity
					// we need to rework this to take slices of components, determined outside the loop
					// so optimal iteration can occur
					let rt = G::iter_as_mut_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		}				
	}

	pub fn iter<F: FnMut(&Iter)>(&self, mut func: F) {
		unsafe {
			let mut it = ecs_filter_iter(self.world, self.filter);
			while ecs_filter_next(&mut it) {
				let iter = Iter::new(&mut it);
				func(&iter);
			}
		}				
	}

}

pub struct FilterBuilder<'w> {
	world: &'w World,
	desc: ecs_filter_desc_t,
	next_term_index: usize,
}

impl<'w> TermBuilder for FilterBuilder<'w> {
    fn world(&mut self) -> *mut ecs_world_t {
        self.world.raw()
    }

	fn filter_desc(&mut self) -> &mut ecs_filter_desc_t {
        &mut self.desc
	}

    fn current_term(&mut self) -> &mut ecs_term_t {
        &mut self.desc.terms[self.next_term_index]
    }

    fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}

impl<'w> FilterBuilder<'w> {
	pub fn new(world: &'w World) -> Self {
		Self { 
			world,
			desc: unsafe { MaybeUninit::zeroed().assume_init() },
			next_term_index: 0
		}
	}

	pub fn with_components<'c, G: ComponentGroup<'c>>(mut self) -> Self {
		G::populate(&mut self);
		self
	}

	pub fn build(self) -> Filter {
		let filter = unsafe { ecs_filter_init(self.world.raw(), &self.desc) };
		Filter { 
			world: self.world.raw(), 
			filter 
		}
	}
}

pub struct FilterGroup<'c, G: ComponentGroup<'c>> {
	world: &'c World,
	filter: *mut ecs_filter_t,
	_phantom: std::marker::PhantomData<G>,
}

// TODO - need to support generalized API via tuples or something
impl<'c, G: ComponentGroup<'c>> FilterGroup<'c, G> {
	pub fn new(world: &'c World) -> Self {
		let world_raw = world.raw();
		let mut desc: ecs_filter_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
		unsafe { G::fill_descriptor(world_raw, &mut desc) };

		let filter = unsafe { ecs_filter_init(world_raw, &desc) };
		FilterGroup { 
			world, 
			filter,
			_phantom: Default::default(),
		}
	}

	pub fn each(&self, mut cb: impl FnMut(Entity, G::RefTuple)) {
		let world_raw = self.world.raw();
		// println!("each - filter: {}, {}, {}", f.term_cache_used, f.terms as u64, f.term_cache.as_ptr() as u64);
		unsafe {
			let mut it = ecs_filter_iter(world_raw, self.filter);
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
		// println!("each_mut - filter: {}, {}, {}", f.term_cache_used, f.terms as u64, f.term_cache.as_ptr() as u64);
		unsafe {
			let mut it = ecs_filter_iter(world_raw, self.filter);
			while ecs_filter_next(&mut it) {
				// Iterate all entities for the type
				for i in 0..it.count {
                    let eid = it.entities.offset(i as isize).as_ref().unwrap();
                    let e = Entity::new(world_raw, *eid);

					// TODO - performance is poor here due to looking up terms for each tuple entry * each entity
					// we need to rework this to take slices of components, determined outside the loop
					// so optimal iteration can occur
					let rt = G::iter_as_mut_tuple(&it, i as isize);
					cb(e, rt);
				}
			}
		}				
	}
}
