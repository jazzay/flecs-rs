use flecs::*;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}

struct Star { }
struct Planet { }
struct Moon { }

fn iterate_tree(e: Entity, parent_pos: Position) {
    // Print hierarchical name of entity & the entity type
    println!("{} [{}]", e.path(), e.type_info().to_str());

    // Get entity position
    let p = e.get::<Position>();

    // Calculate actual position
    let p_actual = Position { x: p.x + parent_pos.x, y: p.y + parent_pos.y};
    println!("{{ {}, {} }}", p_actual.x, p_actual.y);

    // Iterate children recursively
    e.children(|child| {
        iterate_tree(child, p_actual);
    });
}

fn main() {
	println!("Entity Hierarchy starting...");

	let mut world = World::new();

    // We have to manually register all components
	world.component::<Position>();
	world.component::<Star>();
	world.component::<Planet>();
	world.component::<Moon>();

    // Create a simple hierarchy.
    // Hierarchies use ECS relations and the builtin flecs::ChildOf relation to
    // create entities as children of other entities.

    let sun = world.entity().named("Sun")
        .add::<Star>()
        .set::<Position>(Position { x: 1.0, y: 1.0 });

        world.entity().named("Mercury")
            .child_of(sun) // Shortcut for add(flecs::ChildOf, sun)
            .add::<Planet>()
            .set::<Position>(Position { x: 1.0, y: 1.0 });

        world.entity().named("Venus")
            .child_of(sun)
            .add::<Planet>()
            .set::<Position>(Position { x: 2.0, y: 2.0 });

        let earth = world.entity().named("Earth")
            .child_of(sun)
            .add::<Planet>()
            .set::<Position>(Position { x: 3.0, y: 3.0 });

            let moon = world.entity().named("MoonE")
                .child_of(earth)
                .add::<Moon>()
                .set::<Position>(Position { x: 0.1, y: 0.1 });

    // Is the Moon a child of Earth?
    println!("Child of Earth? {}", moon.is_child_of(earth));

    // Do a depth-first walk of the tree
    iterate_tree(sun, Position::default());
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_entity_hierarchy() {
		super::main();
	}
}