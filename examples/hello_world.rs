use flecs::*;

// TODO
// - Add Generic component API to Systems

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Velocity {
	x: f32,
	y: f32,
}

// Tag types
struct Eats { }
struct Apples { }

fn main() {
	println!("Hello World starting...");

	let mut world = World::new();

    // We have to manually register all components
	world.component_named::<Position>("Position");
	world.component_named::<Velocity>("Velocity");
	world.component::<Eats>();
	world.component::<Apples>();

	world.system()
		.signature("Position, Velocity")
		.iter(|it| {
            println!("system_one: entities = {}", it.count());

            let positions = it.term::<Position>(1);
            let vels = it.term::<Velocity>(2);
        
            for index in 0..it.count() {
                let pos = positions.get(index);
                let vel = vels.get(index);
                println!("   {:?}, {:?}", pos, vel);
            }        
        });

    // Register system
    // ecs.system<Position, Velocity>()
    //     .each([](Position& p, Velocity& v) {
    //         p.x += v.x;
    //         p.y += v.y;
    //     });

    // Create an entity with name Bob, add Position and food preference
    let bob = world.entity().named("Bob")
        .set(Position { x: 0.0, y: 0.0 })
        .set(Velocity { x: 1.0, y: 2.0 })
        .add_relation::<Eats, Apples>();

    // Show us what you got
    println!("{}'s got: [ {} ]", bob.name(), bob.type_info().to_str());

    // Run systems twice. Usually this function is called once per frame
    world.progress(0.033);
    world.progress(0.033);

    // See if Bob has moved (he has)
    let p = bob.get::<Position>();
    println!("Bob's position is {{ {}, {} }}", p.x, p.y);
}
