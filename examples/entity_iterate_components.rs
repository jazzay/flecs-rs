use flecs::*;

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

// Tag
struct Human {}

// Two tags used to create a pair
struct Eats { }
struct Apples { }

fn iterate_components(e: Entity) {
    // 1. The easiest way to print the components is to use type::str
    println!("{}", e.type_info().to_str());

    // 2. To get individual component ids, use entity::each
    let mut i = 0;
    e.each(|id| {
        println!("{}: [{}] {}", i, id.raw(), id.to_str());
        i += 1;
    });
    println!("");

    // // 3. we can also inspect and print the ids in our own way. This is a
    // // bit more complicated as we need to handle the edge cases of what can be
    // // encoded in an id, but provides the most flexibility.
    i = 0;
    e.each(|id| {

        if id.has_role() {
            print!("{}: role: {}, ", i, id.role().role_str());
        }

        if id.is_pair() {
            // If id is a pair, extract & print both parts of the pair
            let rel = id.relation();
            let obj = id.object();
            println!("rel: {}, obj: {}", rel.name(), obj.name());
        } else {
            // Id contains a regular entity. Strip role before printing.
            let e = id.entity();
            println!("{}: entity: {}  [{}]", i, e.name(), e.symbol());
        }
        i += 1;
    });

    println!("");
}

fn main() {
	println!("Entity Iterate Components starting...");

	let mut world = World::new();

    // We have to manually register all components
	world.component::<Position>();
	world.component::<Velocity>();
	world.component::<Human>();
	world.component::<Eats>();
	world.component::<Apples>();

    let bob = world.entity()
        .set::<Position>(Position {x: 10.0, y: 20.0 })
        .set::<Velocity>(Velocity {x: 1.0 , y: 1.0 })
        .add::<Human>()
        .add_relation::<Eats, Apples>();

    println!("\nBob's components");
    iterate_components(bob);

    // We can use the same function to iterate the components of a component
    println!("Position's components");
    iterate_components(world.component::<Position>());
}

// Output:

// Bob's components:
// ecs_type_str: Position,Velocity,Human,(Eats,Apples)

// 0: Position
// 1: Velocity
// 2: Human
// 3: (Eats,Apples)

// 0: entity: Position
// 1: entity: Velocity
// 2: entity: Human
// 3: role: PAIR, rel: Eats, obj: Eats


// Position's components:
// ecs_type_str: EcsComponent,(Identifier,Name),(Identifier,Symbol),(OnDelete,Throw)

// 0: Component
// 1: (Identifier,Name)
// 2: (Identifier,Symbol)
// 3: (OnDelete,Throw)

// 0: entity: Component
// 1: role: PAIR, rel: Identifier, obj: Identifier
// 2: role: PAIR, rel: Identifier, obj: Identifier
// 3: role: PAIR, rel: OnDelete, obj: OnDelete
