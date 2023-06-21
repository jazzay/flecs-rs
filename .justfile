default: fmt build test

show:
  just --list

# Builds all binaries
build:
  cargo build --release

# Run all tests
test:
  cargo test

# Formats all code to our chosen style
fmt:
  cargo fmt

# Run all benches
bench:
  cargo bench

# Generates rust binding for Flecs C api
bindings:
  cargo build --features export_bindings
