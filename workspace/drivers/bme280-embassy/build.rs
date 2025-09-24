fn main() {
    // Library crate - only need linker args for examples (no binary targets)
    println!("cargo:rustc-link-arg-examples=-Tlinkall.x");
}