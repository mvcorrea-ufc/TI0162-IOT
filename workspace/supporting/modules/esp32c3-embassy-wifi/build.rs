fn main() {
    // Add linker arguments for examples
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}