use flecs::*;

#[derive(Default, Debug, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}

#[derive(Default, Debug, PartialEq)]
struct Velocity {
	x: f32,
	y: f32,
}

fn main() {
	println!("Prefabs example starting...");

	let mut world = World::new();
	world.component::<Position>();
	world.component::<Velocity>();

	let item1 = world.prefab("Item1")
		.set(Position { x: 1.0, y: 2.0 })
		.set(Velocity { x: 0.0, y: 0.0 });

	let e1 = world.entity()
		.is_a(item1);

	let e2 = world.entity()
		.is_a(item1)
		.set(Velocity { x: 3.0, y: 6.0 });

	let pos = e1.get::<Position>();
	println!("E1 - Position = {:?}", pos);

	let vel = e1.get::<Velocity>();
	println!("E1 - Velocity = {:?}", vel);

	let vel = e2.get::<Velocity>();
	println!("E2 - Velocity = {:?}", vel);
}

// We can also run these within tests. Need to figure out best org
//
#[cfg(test)]
mod tests {
    #[test]
    fn flecs_prefabs() {
	}
}
