use criterion::*;

const ITER_COUNT: usize = 100_000;

mod bevy_bench {
	use super::*;
	use bevy_ecs::prelude::*;

	use bevy_ecs::prelude::Component;

	#[derive(Component, Copy, Clone, Default, Debug, PartialEq)]
	struct Position {
		x: f32,
		y: f32,
		z: f32,
	}

	#[derive(Component, Copy, Clone, Default, Debug, PartialEq)]
	struct Rotation {
		x: f32,
		y: f32,
		z: f32,
	}

	#[derive(Component, Copy, Clone, Default, Debug, PartialEq)]
	struct Velocity {
		x: f32,
		y: f32,
		z: f32,
	}

	fn insert_entities(count: usize) -> World {
		let mut world = World::new();
		for _ in 0..count {
			let mut entity = world.spawn_empty();
			entity.insert(Position::default());
			entity.insert(Rotation::default());
			entity.insert(Velocity { x: 1.0, y: 1.0, z: 1.0 });
		}
		world
	}

	pub struct SimpleInsert;

	impl SimpleInsert {
		pub fn new() -> Self {
			Self
		}
	
		pub fn run(&mut self) {
			insert_entities(ITER_COUNT);
		}
	
		pub fn run_batched(&mut self) {
			let mut world = World::new();
			world.spawn_batch((0..ITER_COUNT).map(|_| {
				(
					Position::default(),
					Rotation::default(),
					Velocity::default(),
				)
			}));
		}
	}	

	pub struct SimpleIter(World);

	impl SimpleIter {
		pub fn new() -> Self {
			let mut world = World::new();
			world.spawn_batch((0..ITER_COUNT).map(|_| {
				(
					Position::default(),
					Rotation::default(),
					Velocity { x: 1.0, y: 1.0, z: 1.0 },
				)
			}));
	
			Self(world)
		}
	
		pub fn run(&mut self) {
			let mut query = self.0.query::<(&Velocity, &mut Position)>();
	
			for (velocity, mut position) in query.iter_mut(&mut self.0) {
				position.x += velocity.x;
				position.y += velocity.y;
				position.z += velocity.z;
			}
		}
	}	
}

mod hecs_bench {
	use super::*;
	use hecs::*;

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Position {
		x: f32,
		y: f32,
		z: f32,
	}

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Rotation {
		x: f32,
		y: f32,
		z: f32,
	}

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Velocity {
		x: f32,
		y: f32,
		z: f32,
	}

	fn insert_entities(count: usize) -> World {
		let mut world = World::new();
		for _ in 0..count {
			let entity = world.spawn(());
			world.insert_one(entity, Position::default()).unwrap();
			world.insert_one(entity, Rotation::default()).unwrap();
			world.insert_one(entity, Velocity { x: 1.0, y: 1.0, z: 1.0 }).unwrap();
		}
		world
	}

	pub struct SimpleInsert;

	impl SimpleInsert {
		pub fn new() -> Self {
			Self
		}
	
		pub fn run(&mut self) {
			insert_entities(ITER_COUNT);
		}

		pub fn run_batched(&mut self) {
			let mut world = World::new();
			world.spawn_batch((0..ITER_COUNT).map(|_| {
				(
					Position::default(),
					Rotation::default(),
					Velocity::default(),
				)
			}));
		}
	}	

	pub struct SimpleIter(World);

	impl SimpleIter {
		pub fn new() -> Self {
			let mut world = World::new();
			world.spawn_batch((0..ITER_COUNT).map(|_| {
				(
					Position::default(),
					Rotation::default(),
					Velocity { x: 1.0, y: 1.0, z: 1.0 },
				)
			}));
	
			Self(world)
		}
	
		pub fn run(&mut self) {
			for (_, (velocity, position)) in self.0.query_mut::<(&Velocity, &mut Position)>() {
				position.x += velocity.x;
				position.y += velocity.y;
				position.z += velocity.z;
			}
		}
	}
}

mod flax_bench {
	use super::*;
	use flax::*;

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Vec3 {
		x: f32,
		y: f32,
		z: f32,
	}
	
	component! {
		pos: Vec3,
		rot: Vec3,
		vel: Vec3,
	}

	fn insert_entities(count: usize) -> World {
		let mut world = World::new();
		for _ in 0..count {
			EntityBuilder::new()
				.set_default(pos())
				.set_default(rot())
				.set(vel(), Vec3 { x: 1.0, y: 1.0, z: 1.0})
				.spawn(&mut world);
		}
		world
	}

	pub struct SimpleInsert;

	impl SimpleInsert {
		pub fn new() -> Self {
			Self
		}
	
		pub fn run(&mut self) {
			insert_entities(ITER_COUNT);
		}

		pub fn run_batched(&mut self) {
			let mut world = World::new();
			let mut batch = BatchSpawn::new(ITER_COUNT);
			
			batch
				.set(pos(), (0..).map(|_| { Vec3::default() }))
				.expect("Invalid length");

			batch
				.set(rot(), (0..).map(|_| { Vec3::default() }))
				.expect("Invalid length");

			batch
				.set(vel(), (0..).map(|_| { Vec3 { x: 1.0, y: 1.0, z: 1.0} }))
				.expect("Invalid length");
				
			batch.spawn(&mut world);

		}
	}	

	pub struct SimpleIter {
		world: World,
		query: Query<(Mutable<Vec3>, Component<Vec3>)>,
	}

	impl SimpleIter {
		pub fn new() -> Self {
			let world = insert_entities(ITER_COUNT);

			// with flax we can cache the query easily 
			let query = Query::new((pos().as_mut(), vel()));
			Self {
				world,
				query
			}
		}
	
		pub fn run(&mut self) {
			for (pos, vel) in &mut self.query.borrow(&self.world) {
				*pos = Vec3 { x: pos.x + vel.x, y: pos.y + vel.y, z: pos.z + vel.z };
			}
		}
	}
}

mod flecs_bench {
	use super::*;
	use flecs::*;

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Position {
		x: f32,
		y: f32,
		z: f32,
	}

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Rotation {
		x: f32,
		y: f32,
		z: f32,
	}

	#[derive(Copy, Clone, Default, Debug, PartialEq)]
	struct Velocity {
		x: f32,
		y: f32,
		z: f32,
	}

	fn insert_entities(count: usize) -> World {
		let mut world = World::new();
		world.component::<Position>();
		world.component::<Rotation>();
		world.component::<Velocity>();

		for _ in 0..count {
			world.entity()
				.set(Position::default())
				.set(Velocity::default())
				.set(Velocity { x: 1.0, y: 1.0, z: 1.0 });
		}
		world
	}

	pub struct SimpleInsert;

	impl SimpleInsert {
		pub fn new() -> Self {
			Self
		}

		pub fn run(&mut self) {
			insert_entities(ITER_COUNT);
		}

		// TODO
		pub fn _run_batched(&mut self) {
		}
	}	

	pub struct SimpleIter(World);

	impl SimpleIter {
		pub fn new() -> Self {
			let world = insert_entities(ITER_COUNT);
			Self(world)
		}
	
		pub fn run_each(&mut self) {
			// while more friendly to the user, this is 20x slower compared to iter()
			//		until we can get some performance improvements related to Tuples use.
			let filter = self.0.filter::<(Position, Velocity)>();
			filter.each_mut(|_e, (position, velocity)| {
				position.x += velocity.x;
				position.y += velocity.y;
				position.z += velocity.z;
			});
		}

		pub fn run_iter(&mut self) {
			let f = self.0.filter_builder().with_components::<(Position, Velocity)>().build();
			f.iter(|it| {
				let positions = it.field::<Position>(1);
				let velocities = it.field::<Velocity>(2);

				for i in 0..it.count() {
					let p = positions.get_mut(i);
					let v = velocities.get(i);
					p.x += v.x;
					p.y += v.y;
					p.z += v.z;
				}
			});
		}
	}
}

fn bench_simple_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_insert");
    group.bench_function("bevy_single", |b| {
        let mut bench = bevy_bench::SimpleInsert::new();
        b.iter(move || bench.run());
    });
    group.bench_function("bevy_batched", |b| {
        let mut bench = bevy_bench::SimpleInsert::new();
        b.iter(move || bench.run_batched());
    });
    group.bench_function("hecs_single", |b| {
        let mut bench = hecs_bench::SimpleInsert::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs_batched", |b| {
        let mut bench = hecs_bench::SimpleInsert::new();
        b.iter(move || bench.run_batched());
    });
    group.bench_function("flax_single", |b| {
        let mut bench = flax_bench::SimpleInsert::new();
        b.iter(move || bench.run());
    });
    group.bench_function("flax_batched", |b| {
        let mut bench = flax_bench::SimpleInsert::new();
        b.iter(move || bench.run_batched());
    });
    group.bench_function("flecs", |b| {
        let mut bench = flecs_bench::SimpleInsert::new();
        b.iter(move || bench.run());
    });
}

fn bench_simple_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_iter");
    group.bench_function("bevy", |b| {
        let mut bench = bevy_bench::SimpleIter::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs", |b| {
        let mut bench = hecs_bench::SimpleIter::new();
        b.iter(move || bench.run());
    });
    group.bench_function("flax", |b| {
        let mut bench = flax_bench::SimpleIter::new();
        b.iter(move || bench.run());
    });
    group.bench_function("flecs_each", |b| {
        let mut bench = flecs_bench::SimpleIter::new();
        b.iter(move || bench.run_each());
    });
    group.bench_function("flecs_iter", |b| {
        let mut bench = flecs_bench::SimpleIter::new();
        b.iter(move || bench.run_iter());
    });
}

criterion_group!(
    benchmarks,
    bench_simple_insert,
    bench_simple_iter,
);
criterion_main!(benchmarks);