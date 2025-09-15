// Build script for ESP32-C3 WiFi Embassy module
// Based on wifi-simple-embassy build configuration

fn main() {
    // Add linker arguments for examples and binaries
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}