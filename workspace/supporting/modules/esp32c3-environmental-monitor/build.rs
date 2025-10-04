fn main() {
    // Set linker arguments for ESP32-C3 (same as blinky)
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
}