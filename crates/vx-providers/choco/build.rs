fn main() {
    // Re-run this build script whenever provider.star changes.
    // This ensures that `include_str!("../provider.star")  in lib.rs always
    // reflects the latest content and that Cargo rebuilds the crate when the
    // Starlark provider definition is updated.
    println!("cargo:rerun-if-changed=provider.star");
}
