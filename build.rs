use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=flecs.h");
    println!("cargo:rerun-if-changed=flecs.c");

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("flecs.h")
		.generate_comments(false)
		.layout_tests(false)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // We generate bindings to an actual source file so that we get better IDE integration
    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("src/bindings.rs"))
        .expect("Couldn't write bindings!");

    // Compile flecs C right into our Rust crate
	cc::Build::new()
		.file("flecs.c")
		.compile("flecs");		
}