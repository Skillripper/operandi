//! Finds the private specs checkout for the spec-backed tests
//! (`build_specs.rs`).
include!("build_specs.rs");

fn main() {
    find_private_specs();
}
