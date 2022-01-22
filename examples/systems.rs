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
		world.entity()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 });
	}

	for _ in 0..(count / 3) {
		world.entity()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 })
			.set(Scale { x: 1.0, y: 1.0 });
	}
}

fn system_one(it: &Iter) {
	println!("system_one: entities = {}", it.count());

	let positions = it.term::<Position>(1);
	let vels = it.term::<Velocity>(2);

	for index in 0..it.count() {
		let pos = positions.get(index);
		let vel = vels.get(index);
		println!("   {:?}, {:?}", pos, vel);
	}
}

fn system_two(it: &Iter) {
	println!("system_two: entities = {}", it.count());

	let positions = it.term::<Position>(1);
	let scales = it.term::<Scale>(2);

	for index in 0..it.count() {
		let pos = positions.get(index);
		let s = scales.get(index);
		println!("   {:?}, {:?}", pos, s);
	}
}

fn main() {
	println!("Systems example starting...");

	let mut world = World::new();
	world.component_named::<Position>("Position");
	world.component_named::<Velocity>("Velocity");
	world.component_named::<Scale>("Scale");

	create_some_entities(&mut world, 3);

	world.system().name("system_one")
		.signature("Position, Velocity, !Scale")
		.iter(system_one);

	world.system().name("system_two")
		.signature("Position, Scale")
		.iter(system_two);

	for _ in 0..5 {
		world.progress(0.033);
	}
}

// We can also run these within tests. Need to figure out best org
//
#[cfg(test)]
mod tests {
    #[test]
    fn flecs_systems() {
	}
}
