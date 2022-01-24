use std::alloc::Layout;
use flecs::*;

#[derive(Default, Debug, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}

const DATA: &str = "test.Data";
type DataValue = [u8; 6];

fn run_data_example(count: i32) {
	let mut world = World::new();

	// regular components can be mixed with dynamic
	world.component_named::<Position>("Position");
	world.component_dynamic(DATA, Layout::new::<DataValue>());

	for _ in 0..count {
		world.entity()
			.set(Position::default())
			.set_dynamic(DATA, &[8; 6]);
	}

	/* Dynamic component systems broken with latest API changes
	world.system::<(Position, _)>().named("DynamicSystem")
		.signature("Position, test.Data")
		.iter(|it| {
			let positions = it.term::<Position>(1);
			let datas = it.get_term_dynamic(2);

			println!("Dynamic System results:");
			for index in 0..it.count() {
				let pos = positions.get(index);
				let data = datas.get(index);
				println!("   {:?}, {:?}", pos, data);
			}
		
		});
	*/
	
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
		super::run_data_example(10);
	}
}