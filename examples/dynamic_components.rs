use std::alloc::Layout;
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

const DATA: &str = "test.Data";
type DataValue = [u8; 6];

fn run_data_example(count: i32) {
	let mut world = World::new();

	// regular components can be mixed with dynamic
	world.component_named::<Position>("Position");
	world.component::<Velocity>();
	let data_comp = world.component_dynamic(DATA, Layout::new::<DataValue>());

	for i in 0..count {
		world.entity()
			.set(Position::default())
			.set(Velocity::default())
			.set_dynamic(DATA, &[i as u8; 6]);
	}

	// Can create filters that mix dynamic and static component types
	let filter = world.filter_builder()
		.term::<Position>()
		.term_dynamic(data_comp)
		.build();

	filter.iter(|it| {
		let positions = it.field::<Position>(1);
		let datas = it.field_dynamic(2);

		println!("Filter 1 result batch:");
		for index in 0..it.count() {
			let pos = positions.get(index);
			let data = datas.get(index);
			println!("   {:?}, {:?}", pos, data);
		}
	});

	// Can create filters that mix dynamic and static component types
	let filter = world.filter_builder()
		.with_components::<(Position, Velocity)>()
		.term_dynamic(data_comp)
		.build();

	filter.iter(|it| {
		let positions = it.field::<Position>(1);
		let velocities = it.field::<Velocity>(2);
		let datas = it.field_dynamic(3);

		println!("Filter 2 result batch:");
		for index in 0..it.count() {
			let pos = positions.get(index);
			let vel = velocities.get(index);
			let data = datas.get(index);
			println!("   {:?}, {:?}, {:?}", pos, vel, data);
		}
	});

	// Can create a system that can iterate static and dynamic components
	world.system().named("DynamicSystem")
		.with_components::<(Position, Velocity)>()
		.term_dynamic(data_comp)
		.iter(|it| {
			let positions = it.field::<Position>(1);
			let datas = it.field_dynamic(3);

			println!("Dynamic System results:");
			for index in 0..it.count() {
				let pos = positions.get(index);
				let data = datas.get(index);
				println!("   {:?}, {:?}", pos, data);
			}
		
		});
		
	world.progress(0.0333);
}

fn main() {
	println!("Dynamic components example starting...");
	run_data_example(8);
}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_dynamic_components() {
		super::main();
	}
}