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

## Compiling with WebAssembly
Compiling with WebAssembly requires using the `wasm32-unknown-emscripten` target.

This is because there is no official toolchain for C/C++ targeting `wasm32-unknown-unknown`, which means that C/C++ bindings with do not work with this target and will result in unresolved symbols.

The Emscripten target which is not as well supported as the `wasm32-unknown-unknown`, which means that `wasm-bindgen` and some other popular Rust libraries that target WebAssembly will not work. So be sure to keep that in mind.

Create a directory to be statically served:
```bash
mkdir static
```

Create a file called `index.html` unders `static/`:
```html
<html>
  <body>
    <script type="module">
      import init from './systems.js'
      await init()
    </script>
  </body>
</html>
```

Build WASM binary:
```bash
build --example systems --target wasm32-unknown-emscripten
cp ./target/wasm32-unknown-emscripten/debug/examples/systems.js ./static/
cp ./target/wasm32-unknown-emscripten/debug/examples/systems.wasm ./static/
```

Serve static file locally with any HTTP server tool, for example `basic-http-server`:
```bash
cargo install basic-http-server
basic-http-server ./static/
```