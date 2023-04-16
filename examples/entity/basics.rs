use flecs::*;

#[derive(Default, Debug, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}

#[derive(Default, Debug, PartialEq)]
struct Walking { }

fn main() {
	println!("Entity Basics starting...");

	let mut world = World::new();

    // We have to manually register all components
	world.component::<Position>();
	world.component::<Walking>();

    // Create an entity with name Bob
    let bob = world.entity().named("Bob")
        // The set operation finds or creates a component, and sets it.
        // Components are automatically registered with the world.
        .set::<Position>(Position { x: 10.0, y: 20.0 }) 
        // The add operation adds a component without setting a value. This is
        // useful for tags, or when adding a component with its default value.
        .add::<Walking>();

    // Get the value for the Position component
    let pos = bob.get::<Position>();
    println!("Bob position: {}, {}", pos.x, pos.y);

    // Overwrite the value of the Position component
    bob.set::<Position>(Position { x: 20.0, y: 30.0 });
    println!("Bob position: {}, {}", pos.x, pos.y);

    // Create another named entity
    let alice = world.entity().named("Alice")
        .set::<Position>(Position { x: 10.0, y: 20.0 });

    // Add a tag after entity is created
    alice.add::<Walking>();

    // Print all of the components the entity has. This will output:
    //    Position, Walking, (Identifier,Name)
    println!("Alice type = [ {} ]", alice.type_info().to_str());

    // Remove tag
    alice.remove::<Walking>();

    // Iterate all entities with Position
    world.each1(|e: flecs::Entity, p: &Position| {
        println!("{}: {}, {}", e.name(), p.x, p.y);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_entity_basics() {
		super::main();
	}
}