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
const RUST_ECS_ROLE_MASK: u64 = 0xFF << 56;	// (ECS_ROLE_MASK as u64)

fn ecs_entity_t_lo(value: u64) -> u32 { value as u32 }
fn ecs_entity_t_hi(value: u64) -> u32 { (value >> 32) as u32 }

fn ecs_pair_relation(e: u64) -> u64 {
	(ecs_entity_t_hi(e & ECS_COMPONENT_MASK)) as u64
}

fn ecs_pair_object(e: u64) -> u64 {
	(ecs_entity_t_lo(e)) as u64
}

impl Id {
	pub(crate) fn new(world: *mut ecs_world_t, id: ecs_id_t) -> Self {
		Self { id, world }
	}

	pub fn raw(&self) -> u64 {
		self.id
	}

	pub fn is_pair(&self) -> bool {
		unsafe { (self.id & RUST_ECS_ROLE_MASK) == ECS_PAIR }
	}

    /* Test if id is a wildcard */
	pub fn is_wildcard(&self) -> bool {
		unsafe { ecs_id_is_wildcard(self.id) }
	}

	pub fn entity(&self) -> Entity {
		assert!(!self.is_pair());
		assert!(!self.has_role());
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
    pub fn has_role(&self) -> bool {
		assert!(RUST_ECS_ROLE_MASK == (ECS_ROLE_MASK as u64));
        return (self.id & RUST_ECS_ROLE_MASK) != 0;
    }

    pub fn role(&self) -> Id {
		Id::new(self.world, self.id & (ECS_ROLE_MASK as u64))
	}

	// from base id type, which don't exist in rust
    pub fn to_str(&self) -> &str {
		let id_str = unsafe { ecs_id_str(self.world, self.id) };
		unsafe { flecs_to_rust_str(id_str) }
    }

    pub fn role_str(&self) -> &str {
		let role_str = unsafe { ecs_role_str(self.id & (ECS_ROLE_MASK as u64)) };
		unsafe { flecs_to_rust_str(role_str) }
    }
}