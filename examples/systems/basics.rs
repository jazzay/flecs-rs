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
    world.component::<Position>();
    world.component::<Velocity>();

    // Can wire a function into a system
    let system = world
        .system()
        .expr("Position, Velocity")
        .each_mut::<(Position, Velocity)>(|e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
            println!("{}: {:?}, {:?}", e.name(), p, v);
        });

    world
        .entity()
        .named("e1")
        .set::<Position>(Position { x: 10.0, y: 20.0 })
        .set::<Velocity>(Velocity { x: 1.0, y: 2.0 });

    world
        .entity()
        .named("e2")
        .set::<Position>(Position { x: 10.0, y: 20.0 })
        .set::<Velocity>(Velocity { x: 3.0, y: 4.0 });

    // This entity will not match as it does not have Position, Velocity
    world
        .entity()
        .named("e3")
        .set::<Position>(Position { x: 10.0, y: 20.0 });

    system.run(0.333);
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_systems_basics() {
        super::main();
    }
}
