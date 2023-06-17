use flecs_api::*;

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
	let mut world = World::new();

	// Must register our components
	world.component::<Position>();
	world.component::<Velocity>();

    // Create a query for Position, Velocity. Queries are the fastest way to
    // iterate entities as they cache results.
    let mut q = world.query()
		.with_components::<(Position, Velocity)>()
		.build();
		
    // Create a few test entities for a Position, Velocity query
	world.entity().named("e1")
		.set(Position { x: 10.0, y: 20.0 })
		.set(Velocity { x: 1.0, y: 2.0 });

	world.entity().named("e2")
		.set(Position { x: 10.0, y: 20.0 })
		.set(Velocity { x: 3.0, y: 4.0 });

    // This entity will not match as it does not have Position, Velocity
	world.entity().named("e3")
		.set(Position { x: 10.0, y: 20.0 });

	// The each() function iterates each entity individually and accepts an
    // entity argument plus arguments for each query component as a tuple:
    q.each_mut::<(Position, Velocity)>(|e, (p, v)| {
        p.x += v.x;
        p.y += v.y;
		println!("Each - {}: {:?}, {:?}", e.name(), p, v);
    });

    // iter() is a bit more verbose, but allows for more control over how entities
    // are iterated as it provides multiple entities in the same callback.
	q.iter(|it| {
		let positions = it.field::<Position>(1);
		let vels = it.field::<Velocity>(2);
	
		for index in 0..it.count() {
			let pos = positions.get(index);
			let vel = vels.get(index);
			println!("Iter - {:?}, {:?}", pos, vel);
		}	
	});
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_queries_basics() {
		super::main();
	}
}