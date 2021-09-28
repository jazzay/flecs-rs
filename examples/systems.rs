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

fn create_some_entities(world: &mut World, count: usize) {
	for _ in 0..count {
		world.entity_builder()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 })
			.build();
	}
}

fn main() {
	println!("Systems example starting...");

	let mut world = World::new();
	world.component_named::<Position>("Position");
	world.component_named::<Velocity>("Velocity");

	create_some_entities(&mut world, 3);

	let sys = world.system()
		.name("HelloJason")
		.signature("Position, Velocity")
		.iter(|it| {
			println!("my system was called: {}", it.count());

			let positions = it.term::<Position>(1);
			let vels = it.term::<Velocity>(2);

			for index in 0..it.count() {
				let pos = positions.get(index);
				let vel = vels.get(index);
				println!("   {:?}, {:?}", pos, vel);
			}
		});

	// sys.interval(1.0);

	for _ in 0..10 {
		sys.run(0.033);
	}
}

// We can also run these within tests. Need to figure out best org
//
#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn flecs_systems() {
	}
}