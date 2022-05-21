use std::env;
use std::path::PathBuf;

const EM_OS: &str = "emscripten";

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=flecs.h");
    println!("cargo:rerun-if-changed=flecs.c");
    println!("cargo:rerun-if-changed=build.rs");

    // Grab this value because #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))] does not work in build.rs
    // because it assumes that the target is the default OS target
    // when you specify wasm32-unknown-emscripten.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap().to_string();
    if target_os == EM_OS {
      // Export as JS file as ES6 Module by adding emscripten flag
      println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
      println!("cargo:rustc-link-arg=-sMODULARIZE=1");
    }

    // Standard library include path
    let mut include_path = env::var("STDLIB").unwrap_or("/usr/include".to_string()).to_string();
    let include_flag = String::from("-I") + &include_path[..include_path.len()];

    println!(include_flag);

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("flecs.h")
        // Nessecary for Emscripten target.
        .clang_arg("-fvisibility=default")
        .clang_arg(include_flag)
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