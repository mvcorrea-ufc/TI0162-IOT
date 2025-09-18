// Build script for ESP32-C3 WiFi Embassy module
// Based on wifi-simple-embassy build configuration

fn main() {
    // Add linker arguments for examples (library only module)
    println!("cargo:rustc-link-arg-examples=-Tlinkall.x");
}