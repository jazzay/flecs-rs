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

fn create_some_filters() -> World {
	println!("Filter example starting...");

	let mut world = World::new();
	world.component::<Position>(None);
	world.component::<Velocity>(None);

	world.entity_builder()
		.set(Position { x: 1.0, y: 2.0 })
		.set(Velocity { x: 2.0, y: 4.0 })
		.build();

	world.entity_builder()
		.set(Position { x: 3.0, y: 9.0 })
		.set(Velocity { x: 0.0, y: 10.0 })
		.build();

	world
}

fn tick(world: &mut World) {
	let mut result = [ 0.0, 0.0 ];
	let filter = Filter::new_2::<Position, Velocity>(world.raw());
	filter.each(|pos: &Position, vel: &Velocity| {
		result[0] += pos.x + vel.x;
		result[1] += pos.y + vel.y;
		println!("Iter: {:?}  {:?}", pos, vel);
	});
	assert_eq!(result, [6.0, 25.0]);
}

fn main() {
	let mut world = create_some_filters();
	tick(&mut world);
}

// We can also run these within tests. Need to figure out best org
//
#[cfg(test)]
mod tests {
    #[test]
    fn flecs_filters() {
		let mut world = super::create_some_filters();
		super::tick(&mut world);
		//assert_eq!(result[0], 22.0);
	}
}