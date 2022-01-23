# flecs-rs: Rust bindings for Flecs

A Rust binding for the Flecs ECS library: 
https://github.com/SanderMertens/flecs

## A Simple Example

```rust
use flecs::*;

#[derive(Default, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default, Debug, PartialEq)]
struct Walking { }

fn main() {
    let mut world = World::new();

    // We have to manually register all components
    world.component::<Position>();
    world.component::<Walking>();

    // Create an entity with name Bob
    let bob = world.entity().named("Bob")
        .set(Position { x: 10.0, y: 20.0 }) 
        .add::<Walking>();

    // Get the value for the Position component
    let pos = bob.get::<Position>();
    println!("Bob position: {}, {}", pos.x, pos.y);

    // Overwrite the value of the Position component
    bob.set(Position { x: 20.0, y: 30.0 });
    println!("Bob position: {}, {}", pos.x, pos.y);

    // Create another named entity
    let alice = world.entity().named("Alice")
        .set(Position { x: 10.0, y: 20.0 });

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
```

## Compiling and running the examples

```bash
git clone https://github.com/jazzay/flecs-rs
cd flecs-rs
cargo build --release
```

Main examples are located in the `examples` directory. These can be run like so:

```bash
cargo test

cargo run --example hello_world
cargo run --example prefabs
```
