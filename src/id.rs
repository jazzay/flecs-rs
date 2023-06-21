use crate::*;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Id {
	id: ecs_id_t,
	world: *mut ecs_world_t,
}

impl AsEcsId for Id {
	fn id(&self) -> ecs_id_t {
		self.id
	}
}

// Bindgen creates i64 for some large ull :(
const RUST_ECS_ID_FLAGS_MASK: u64 = ECS_ID_FLAGS_MASK as u64;

fn ecs_entity_t_lo(value: u64) -> u32 {
	value as u32
}
fn ecs_entity_t_hi(value: u64) -> u32 {
	(value >> 32) as u32
}
fn ecs_entity_t_comb(lo: u64, hi: u64) -> u64 {
	(hi << 32) + lo
}

pub fn ecs_pair(pred: u64, obj: u64) -> u64 {
	unsafe { ECS_PAIR | ecs_entity_t_comb(obj, pred) }
}

fn ecs_pair_relation(e: u64) -> u64 {
	(ecs_entity_t_hi(e & ECS_COMPONENT_MASK)) as u64
}

fn ecs_pair_object(e: u64) -> u64 {
	(ecs_entity_t_lo(e)) as u64
}

/* TODO: Review these macro like functions again since v3.0
/* Get object from pair with the correct (current) generation count */
#define ecs_pair_first(world, pair) ecs_get_alive(world, ECS_PAIR_FIRST(pair))
#define ecs_pair_second(world, pair) ecs_get_alive(world, ECS_PAIR_SECOND(pair))
#define ecs_pair_relation ecs_pair_first
#define ecs_pair_object ecs_pair_second
*/

impl Id {
	pub(crate) fn new(world: *mut ecs_world_t, id: ecs_id_t) -> Self {
		Self { id, world }
	}

	pub fn raw(&self) -> u64 {
		self.id
	}

	pub fn is_pair(&self) -> bool {
		unsafe { (self.id & RUST_ECS_ID_FLAGS_MASK) == ECS_PAIR }
	}

	/* Test if id is a wildcard */
	pub fn is_wildcard(&self) -> bool {
		unsafe { ecs_id_is_wildcard(self.id) }
	}

	pub fn entity(&self) -> Entity {
		assert!(!self.is_pair());
		assert!(!self.has_flags());
		Entity::new(self.world, self.id)
	}

	pub fn relation(&self) -> Entity {
		assert!(self.is_pair());

		let e = ecs_pair_relation(self.id);
		if !self.world.is_null() {
			return Entity::new(self.world, unsafe { ecs_get_alive(self.world, e) });
		} else {
			return Entity::new(std::ptr::null_mut(), e);
		}
	}

	pub fn object(&self) -> Entity {
		assert!(self.is_pair());

		let e = ecs_pair_object(self.id);
		if !self.world.is_null() {
			return Entity::new(self.world, unsafe { ecs_get_alive(self.world, e) });
		} else {
			return Entity::new(std::ptr::null_mut(), e);
		}
	}

	/* Test if id has any role */
	pub fn has_flags(&self) -> bool {
		return (self.id & RUST_ECS_ID_FLAGS_MASK) != 0;
	}

	pub fn flags(&self) -> Entity {
		Entity::new(self.world, self.id & RUST_ECS_ID_FLAGS_MASK)
	}

	// from base id type, which don't exist in rust
	pub fn to_str(&self) -> &str {
		let id_str = unsafe { ecs_id_str(self.world, self.id) };
		unsafe { flecs_to_rust_str(id_str) }
	}
}
