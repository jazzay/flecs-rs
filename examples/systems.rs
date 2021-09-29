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

#[derive(Default, Debug, PartialEq)]
struct Scale {
	x: f32,
	y: f32,
}

fn create_some_entities(world: &mut World, count: usize) {
	for _ in 0..count {
		world.entity_builder()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 })
			.set(Scale { x: 1.0, y: 1.0 })
			.build();
	}
}

fn main() {
	println!("Systems example starting...");

	let mut world = World::new();
	world.component_named::<Position>("Position");
	world.component_named::<Velocity>("Velocity");
	world.component_named::<Scale>("Scale");

	create_some_entities(&mut world, 3);

	world.system()
		.name("HelloJason")
		.signature("Position, Velocity")
		.iter(|it| {
			println!("System1: entities = {}", it.count());

			let positions = it.term::<Position>(1);
			let vels = it.term::<Velocity>(2);

			for index in 0..it.count() {
				let pos = positions.get(index);
				let vel = vels.get(index);
				println!("   {:?}, {:?}", pos, vel);
			}
		});

	world.system()
		.name("System2")
		.signature("Position, Scale")
		.iter(|it| {
			println!("System2: entities = {}", it.count());

			let positions = it.term::<Position>(1);
			let scales = it.term::<Scale>(2);

			for index in 0..it.count() {
				let pos = positions.get(index);
				let s = scales.get(index);
				println!("   {:?}, {:?}", pos, s);
			}
		});

	for _ in 0..5 {
		world.progress(0.033);
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

// Next steps:
//
// DONE - Get world progress working + multiple systems
// what other mvp functionality?
// experiment with dynamic types