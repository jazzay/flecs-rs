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

#[derive(Default, Debug, PartialEq)]
struct Scale {
    x: f32,
    y: f32,
}

fn create_some_entities(world: &mut World, count: usize) {
    for i in 0..count {
        world
            .entity()
            .named(&format!("A-{}", i))
            .set(Position { x: 1.0, y: 2.0 })
            .set(Velocity { x: 2.0, y: 4.0 });
    }

    for i in 0..(count / 3) {
        world
            .entity()
            .named(&format!("B-{}", i))
            .set(Position { x: 1.0, y: 2.0 })
            .set(Velocity { x: 2.0, y: 4.0 })
            .set(Scale { x: 1.0, y: 1.0 });
    }
}

// Iter functions not supported for now
fn system_with_iter(it: &Iter) {
    println!("system_with_iter: entities = {}", it.count());

    let positions = it.field::<Position>(1);
    let vels = it.field::<Velocity>(2);

    for index in 0..it.count() {
        let pos = positions.get(index);
        let vel = vels.get(index);
        println!("   {:?}, {:?}", pos, vel);
    }
}

fn system_one(e: Entity, (pos, vel): (&mut Position, &mut Velocity)) {
    pos.x += vel.x;
    pos.y += vel.y;
    println!("Sys1 - {}: {:?}, {:?}", e.name(), pos, vel);
}

fn main() {
    println!("Systems example starting...");

    let mut world = World::new();
    world.component::<Position>();
    world.component::<Velocity>();
    world.component::<Scale>();

    create_some_entities(&mut world, 3);

    // Can wire a function into a system
    world
        .system()
        .named("system_one")
        .expr("Position, Velocity, !Scale")
        .each_mut::<(Position, Velocity)>(system_one);

    // Or pass a closure directly
    world
        .system()
        .named("system_two")
        .expr("Position, Scale")
        .each_mut::<(Position, Scale)>(|e, (pos, s)| {
            println!("Sys2 - {}: {:?}, {:?}", e.name(), pos, s);
        });

    // We don't yet support 1 comp systems yet due to tuple macro impl
    // world.system::<Position>().name("system_three")
    // 	.signature("Position")
    // 	.iter(system_two);

    world
        .system()
        .named("system_with_iter")
        .expr("Position, Velocity")
        .iter(system_with_iter);

    for _ in 0..5 {
        world.progress(0.033);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_systems() {
        super::main();
    }
}
