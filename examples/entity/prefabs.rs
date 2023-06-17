use flecs_api::*;

// TODO
// Update iteration APIs (each, etc) to support mutable component refs

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Attack {
    value: f32,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Defense {
    value: f32,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct FreightCapacity {
    value: f32,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct ImpulseSpeed {
    value: f32,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct HasFTL {}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

fn main() {
    println!("Entity Prefabs starting...");

    let mut world = World::new();

    // We have to manually register all components
    world.component::<Attack>();
    world.component::<Defense>();
    world.component::<FreightCapacity>();
    world.component::<ImpulseSpeed>();
    world.component::<HasFTL>();
    world.component::<Position>();

    // Create a prefab hierarchy. Prefabs are entities that by default are
    // ignored by queries.

    let spaceship = world
        .prefab("Spaceship")
        // Add components to prefab entity as usual
        .set(ImpulseSpeed { value: 50.0 })
        .set(Defense { value: 50.0 })
        // By default components in an inheritance hierarchy are shared between
        // entities. The override function ensures that instances have a private
        // copy of the component.
        .override_component::<Position>();

    let freighter = world
        .prefab("Freighter")
        // Short for .add(flecs::IsA, spaceship). This ensures the entity
        // inherits all components from spaceship.
        .is_a(spaceship)
        .add::<HasFTL>()
        .set(FreightCapacity { value: 100.0 })
        .set(Defense { value: 100.0 });

    let mammoth_freighter = world
        .prefab("MammothFreighter")
        .is_a(freighter)
        .set(FreightCapacity { value: 500.0 })
        .set(Defense { value: 300.0 });

    world
        .prefab("Frigate")
        .is_a(spaceship)
        .add::<HasFTL>()
        .set(Attack { value: 100.0 })
        .set(Defense { value: 75.0 })
        .set(ImpulseSpeed { value: 125.0 });

    // Create a regular entity from a prefab.
    // The instance will have a private copy of the Position component, because
    // of the override in the spaceship entity. All other components are shared.
    let inst = world
        .entity()
        .named("my_mammoth_freighter")
        .is_a(mammoth_freighter);

    // As of Flecs 3.2.1 the 'uninitialized' Position component started coming back with weird values
    // so let's initialize it properly. Perhaps we need to wire up default constructor to Flecs lifecycle soon!
    inst.set(Position { x: 10.0, y: 200.0 });

    // Inspect the type of the entity. This outputs:
    //    Position,(Identifier,Name),(IsA,MammothFreighter)
    println!("Instance type: [{}]", inst.type_info().to_str());

    // Even though the instance doesn't have a private copy of ImpulseSpeed, we
    // can still get it using the regular API (outputs 50)
    let speed = inst.get::<ImpulseSpeed>();
    println!("Impulse speed: {}", speed.value);

    // Prefab components can be iterated just like regular components:
    world
        .filter::<(ImpulseSpeed, Position)>()
        .each(|e: Entity, (_is, p)| {
            // TODO - need to support mutable component access
            //p.x += is.value;
            println!("{}: {{ {}, {} }}", e.name(), p.x, p.y);
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_entity_prefabs() {
        super::main();
    }
}
