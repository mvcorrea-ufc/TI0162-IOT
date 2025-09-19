//! # Error Handling Example
//!
//! This example demonstrates the unified error handling system in the IoT common library.
//! It shows how to create, handle, and convert between different error types while 
//! maintaining no_std compatibility.

#![no_std]
#![no_main]

// In a real embedded application, you would have a panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use iot_common::{
    IoTResult, IoTError, SensorError, NetworkError, HardwareError,
    error::utils, result::IoTResultExt
};
use heapless::String;

/// Example sensor reading function that can fail
fn read_bme280_temperature() -> IoTResult<f32> {
    // Simulate different failure scenarios
    let sensor_status = get_sensor_status();
    
    match sensor_status {
        0 => Ok(25.6), // Success case
        1 => {
            let error = SensorError::I2CError(utils::error_message("I2C timeout"));
            Err(IoTError::sensor(error).with_context("BME280 temperature read"))
        },
        2 => {
            let error = SensorError::InvalidData(utils::error_message("Checksum failed"));
            Err(IoTError::sensor(error).with_context("BME280 data validation"))
        },
        _ => {
            let error = SensorError::NotResponding(utils::error_message("No ACK received"));
            Err(IoTError::sensor(error).with_context("BME280 communication"))
        }
    }
}

/// Example WiFi connection function
fn connect_wifi() -> IoTResult<()> {
    // Simulate WiFi connection process
    let wifi_status = get_wifi_status();
    
    match wifi_status {
        0 => Ok(()),
        1 => {
            let error = NetworkError::WiFiConnectionFailed(
                utils::error_message("Authentication failed")
            );
            Err(IoTError::network(error).with_context("WiFi authentication"))
        },
        2 => {
            let error = NetworkError::DHCPFailed(
                utils::error_message("DHCP server not responding")
            );
            Err(IoTError::network(error).with_context("IP address assignment"))
        },
        _ => {
            let error = NetworkError::HardwareInitFailed(
                utils::error_message("WiFi chip not responding")
            );
            Err(IoTError::network(error).with_context("WiFi hardware initialization"))
        }
    }
}

/// Example hardware initialization function
fn init_gpio_pins() -> IoTResult<()> {
    let gpio_status = get_gpio_status();
    
    if gpio_status != 0 {
        let error = HardwareError::GPIOError(
            utils::error_message("Pin configuration failed")
        );
        return Err(IoTError::hardware(error).with_context("GPIO initialization"));
    }
    
    Ok(())
}

/// Example of chaining operations with error context
fn initialize_system() -> IoTResult<()> {
    // Initialize hardware first
    init_gpio_pins()
        .with_context("System hardware initialization")?;
    
    // Connect to WiFi
    connect_wifi()
        .with_context("Network connectivity setup")?;
    
    // Test sensor communication
    let _temperature = read_bme280_temperature()
        .with_context("Sensor validation")?;
    
    Ok(())
}

/// Example error handling with recovery
fn robust_sensor_reading() -> IoTResult<f32> {
    const MAX_RETRIES: u8 = 3;
    let mut retries = 0;
    
    loop {
        match read_bme280_temperature() {
            Ok(temp) => return Ok(temp),
            Err(e) if retries < MAX_RETRIES => {
                retries += 1;
                // In a real system, you might add a delay here
                continue;
            },
            Err(e) => {
                return Err(e.with_context(&format!(
                    "Failed after {} retries", MAX_RETRIES
                )));
            }
        }
    }
}

/// Example error categorization and handling
fn handle_error_by_type(error: &IoTError) {
    match error.kind() {
        iot_common::error::IoTErrorKind::Sensor(_) => {
            // Handle sensor errors - maybe try alternative sensors
            log_error("SENSOR", error);
        },
        iot_common::error::IoTErrorKind::Network(_) => {
            // Handle network errors - maybe retry connection
            log_error("NETWORK", error);
        },
        iot_common::error::IoTErrorKind::Hardware(_) => {
            // Handle hardware errors - maybe reset hardware
            log_error("HARDWARE", error);
        },
        iot_common::error::IoTErrorKind::Configuration(_) => {
            // Handle config errors - maybe use defaults
            log_error("CONFIG", error);
        },
        iot_common::error::IoTErrorKind::System(_) => {
            // Handle system errors - maybe restart system
            log_error("SYSTEM", error);
        }
    }
}

/// Simple error logging function (would use RTT in real application)
fn log_error(category: &str, error: &IoTError) {
    // In a real embedded application, this would use RTT or UART
    // For now, we just format the error information
    let error_code = error.error_code();
    let _formatted_error = format_error(category, error, error_code);
}

/// Format error for logging/debugging
fn format_error(category: &str, error: &IoTError, code: u16) -> String<128> {
    let mut formatted: String<128> = String::new();
    let _ = formatted.push_str("[");
    let _ = formatted.push_str(category);
    let _ = formatted.push_str("] ");
    
    // Add error code
    let mut code_str: String<8> = String::new();
    let _ = write!(code_str, "{}", code);
    let _ = formatted.push_str(&code_str);
    let _ = formatted.push_str(": ");
    
    // Add error message (truncated to fit)
    let error_msg = format!("{}", error);
    let remaining = formatted.capacity() - formatted.len();
    let msg_slice = if error_msg.len() > remaining {
        &error_msg[..remaining.saturating_sub(3)]
    } else {
        &error_msg
    };
    let _ = formatted.push_str(msg_slice);
    
    if error_msg.len() > remaining {
        let _ = formatted.push_str("...");
    }
    
    formatted
}

/// Example main function showing comprehensive error handling
#[no_mangle]
pub fn main() -> ! {
    // Initialize system with comprehensive error handling
    match initialize_system() {
        Ok(()) => {
            // System initialized successfully
            run_main_loop();
        },
        Err(e) => {
            handle_error_by_type(&e);
            // In a real system, might try recovery or enter safe mode
            enter_error_mode();
        }
    }
}

fn run_main_loop() -> ! {
    loop {
        // Main application logic
        match robust_sensor_reading() {
            Ok(temperature) => {
                process_temperature(temperature);
            },
            Err(e) => {
                handle_error_by_type(&e);
                // Continue operation even if sensor reading fails
            }
        }
        
        // Add delay in real application
        cortex_m::asm::delay(1000);
    }
}

fn enter_error_mode() -> ! {
    // Safe mode - minimal functionality
    loop {
        // Blink error LED or similar
        cortex_m::asm::delay(500);
    }
}

fn process_temperature(temperature: f32) {
    // Process the temperature reading
    let _ = temperature; // Suppress unused warning
}

// Simulation functions for demonstration
fn get_sensor_status() -> u8 { 0 } // Simulate success
fn get_wifi_status() -> u8 { 0 }   // Simulate success  
fn get_gpio_status() -> u8 { 0 }   // Simulate success

// Required for formatting in no_std
use core::fmt::Write;