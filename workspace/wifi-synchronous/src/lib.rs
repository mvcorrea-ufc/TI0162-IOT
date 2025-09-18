#![no_std]
#![doc = include_str!("../README.md")]

//! # WiFi Synchronous
//! 
//! Synchronous WiFi connectivity module for ESP32-C3.
//! 
//! This module provides synchronous (blocking) WiFi connectivity using traditional
//! blocking patterns as an alternative to async Embassy framework.
//! Designed with compatible API concepts from wifi-embassy for easy migration.

extern crate alloc;

pub mod wifi_manager;

// Re-export main types (matches wifi-embassy exports)
pub use wifi_manager::{WiFiManager, WiFiConfig, ConnectionInfo, WiFiError};

// Re-export helper functions for manual usage
pub use wifi_manager::{
    create_interface,
    create_dhcp_socket,
    set_dhcp_hostname,
    create_stack,
    configure_wifi,
    scan_networks,
    wait_for_connection,
    wait_for_ip,
    get_status,
    WiFiConnection, // Legacy compatibility
};

// Simple re-exports for basic WiFi functionality
pub use esp_wifi::wifi::{WifiController, WifiDevice, Configuration, ClientConfiguration};
pub use smoltcp::wire::Ipv4Address;