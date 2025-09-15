fn main() {
    // Only apply linker arguments when building examples
    if std::env::var("CARGO_CFG_EXAMPLE").is_ok() {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
}