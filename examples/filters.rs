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
			.set(Velocity { x: 2.0, y: 4.0 })
			.set(Scale { x: 1.0, y: 0.5 });
	}
}

fn tick(world: &mut World) -> [f32; 2] {
	let mut result = [ 0.0, 0.0 ];
	let filter = Filter::new_1::<Position>(world.raw());
	filter.each_1(|_e, pos: &Position| {
		result[0] += pos.x;
		result[1] += pos.y;
		// println!("Iter: {:?}  {:?}", pos, vel);
	});

	// Component tuples
	let filter = FilterGroup::<(Position, Velocity)>::new(world);
	filter.each(|_e, (pos, vel)| {
		result[0] += pos.x + vel.x;
		result[1] += pos.y + vel.y;
		println!("Group-2 Iter: {:?}  {:?}", pos, vel);
	});

	let filter = world.filter::<(Position, Velocity, Scale)>();
	filter.each_mut(|_e, (pos, vel, s)| {
		pos.x = pos.x + vel.x;
		pos.y = pos.y + vel.y;
		println!("Group-3 Iter: {:?}  {:?}  {:?}", pos, vel, s);
	});

	// Single components not working yet due to macro shenanigans
	// let filter = world.filter::<(Position,)>();
	// filter.each(|_e, (pos, )| {
	// 	println!("Filter Single: {:?}  {:?}  {:?}", pos);
	// });

	// assert_eq!(result, [6.0, 25.0]);
	result
}

fn main() {
	println!("Filter example starting...");

	let mut result = [0.0, 0.0];
	for _ in 0..10 {
		let mut world = World::new();
		world.component::<Position>();
		world.component::<Velocity>();
		world.component::<Scale>();
	
		create_some_entities(&mut world, 5);
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
		world.component::<Scale>();
	
		create_some_entities(&mut world, 1000);
		super::tick(&mut world);
		//assert_eq!(result[0], 22.0);
	}
}