use flecs_api::*;

fn main() {
	let world = World::new();

    // Create system that prints delta_time. This system doesn't query for any
    // components which means it won't match any entities, but will still be ran
    // once for each call to ecs_progress.
	world.system()
		.iter(|it| {
			println!("delta_time: {:?}", it.delta_time());
		});

    // Call progress with 0.0f for the delta_time parameter. This will cause
    // ecs_progress to measure the time passed since the last frame. The
    // delta_time of the first frame is a best guess (16ms).
	world.progress(0.0);

    // The following calls should print a delta_time of approximately 100ms
	//
	std::thread::sleep(std::time::Duration::from_millis(100));
    world.progress(0.0);

	std::thread::sleep(std::time::Duration::from_millis(100));
    world.progress(0.0);

}

#[cfg(test)]
mod tests {
    #[test]
    fn flecs_systems_delta_time() {
		super::main();
	}
}