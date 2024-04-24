#[cfg(feature = "export_bindings")]
const EM_OS: &str = "emscripten";

#[cfg(feature = "export_bindings")]
fn generate_bindings() {
	use std::env;
	use std::path::PathBuf;

	// Grab this value because #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))] does not work in build.rs
	// because it assumes that the target is the default OS target
	// when you specify wasm32-unknown-emscripten.
	let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

	let mut bindings = bindgen::Builder::default()
		.header("flecs.h")
		.clang_arg("-fvisibility=default") // Necessary for Emscripten target.
		.generate_comments(false)
		.layout_tests(false)
		// Tell cargo to invalidate the built crate whenever any of the
		// included header files changed.
		.parse_callbacks(Box::new(bindgen::CargoCallbacks));

	if target_os == EM_OS {
		// Export as JS file as ES6 Module by adding emscripten flag
		println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
		println!("cargo:rustc-link-arg=-sMODULARIZE=1");

		// Standard library include path
		// To support all platforms we should use the emsdk sysroot itself for the include path.
		let emsdk = env::var("EMSDK").unwrap();
		let emsdk_include_path = format!("{}/upstream/emscripten/cache/sysroot/include", emsdk);
		let include_path = env::var("STDLIB").unwrap_or(emsdk_include_path);
		let include_flag = String::from("-I") + &include_path[..include_path.len()];
		println!("Used Include Path: {}", include_path);

		bindings = bindings.clang_arg(include_flag);
	}

	// export comments from flecs source
	let bindings = bindings
		.generate_comments(true)
		.clang_arg("-fparse-all-comments")
		// this yields two small comments
		.clang_arg("-fretain-comments-from-system-headers")
		.parse_callbacks(Box::new(CommentsCallbacks));

	let bindings = bindings
		.allowlist_file("flecs.c")
		.allowlist_file("flecs.h")
		.generate()
		.expect("Unable to generate bindings");

	let crate_root: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
	bindings.write_to_file(crate_root.join("src/bindings.rs")).unwrap();
}

fn main() {
	// Tell cargo to invalidate the built crate whenever the sources change
	println!("cargo:rerun-if-changed=flecs.h");
	println!("cargo:rerun-if-changed=flecs.c");
	println!("cargo:rerun-if-changed=build.rs");

	// if cfg!(feature = "export_bindings") {
	//     generate_bindings();
	// }
	#[cfg(feature = "export_bindings")]
	generate_bindings();

	#[cfg(not(feature = "enable_export_symbols"))]
	const FLECS_EXPORT: &str = "flecs_STATIC";
	#[cfg(feature = "enable_export_symbols")]
	const FLECS_EXPORT: &str = "flecs_EXPORTS";

	// Compile flecs C right into our Rust crate
	cc::Build::new()
		.warnings(true)
		.extra_warnings(true)
		.define(FLECS_EXPORT, None)
		.define("NDEBUG", None)
		// .flag("-flto")			// no impact on Arm. Perhaps useful to other archs.
		// .flag("-fuse-ld=lld")	// not available on MacOS/Arm
		.file("flecs.c")
		.compile("flecs");
}

#[cfg(feature = "export_bindings")]
#[derive(Debug)]
struct CommentsCallbacks;

#[cfg(feature = "export_bindings")]
impl bindgen::callbacks::ParseCallbacks for CommentsCallbacks {
	fn process_comment(&self, comment: &str) -> Option<String> {
		// 1: trimming the comments
		let comment = comment.trim();
		// 2: brackets do not entail intra-links
		let comment = comment.replace("[", "\\[");
		let comment = comment.replace("]", "\\]");

		// ensure all links are padded with < and >
		let url_re = regex::Regex::new(r"(?P<url>https?://[^\s]+)").unwrap();
		let comment = url_re
			.replace_all(comment.as_str(), |caps: &regex::Captures| format!("<{}>", &caps["url"]))
			.into_owned();

		Some(comment)
	}
}
