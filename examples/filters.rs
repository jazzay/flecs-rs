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

// struct Serializable {}

fn create_some_entities(world: &mut World, count: usize) {
	for _ in 0..count {
		world.entity_builder()
			.set(Position { x: 1.0, y: 2.0 })
			.set(Velocity { x: 2.0, y: 4.0 })
			.build();
	}
}

fn tick(world: &mut World) -> [f32; 2] {
	let mut result = [ 0.0, 0.0 ];
	let filter = Filter::new_2::<Position, Velocity>(world.raw());
	filter.each_2(|_e, pos: &Position, vel: &Velocity| {
		result[0] += pos.x + vel.x;
		result[1] += pos.y + vel.y;
		// println!("Iter: {:?}  {:?}", pos, vel);
	});
	// assert_eq!(result, [6.0, 25.0]);
	result
}

fn main() {
	println!("Filter example starting...");

	let mut result = [0.0, 0.0];
	for _ in 0..1000 {
		let mut world = World::new();
		world.component::<Position>();
		world.component::<Velocity>();
	
		create_some_entities(&mut world, 100000);
		result = tick(&mut world);
	}
	println!("Result: {:?}", result);
}

// We can also run these within tests. Need to figure out best org
//
#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn flecs_filters() {
		let mut world = World::new();
		world.component::<Position>();
		world.component::<Velocity>();
	
		create_some_entities(&mut world, 1000);
		super::tick(&mut world);
		//assert_eq!(result[0], 22.0);
	}
}