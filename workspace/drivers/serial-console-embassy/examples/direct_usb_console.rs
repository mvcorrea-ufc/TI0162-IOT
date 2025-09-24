//! Direct USB Serial/JTAG Console for ESP32-C3
//! 
//! Console runs directly on USB Serial/JTAG peripheral
//! Accessible via /dev/ttyACM0 with no bridging needed

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_hal::{
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
    timer::timg::TimerGroup,
    Async,
};
use embedded_io_async::{Read, Write};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[embassy_executor::task]
async fn usb_console_task(mut usb_tx: UsbSerialJtagTx<'static, Async>, mut usb_rx: UsbSerialJtagRx<'static, Async>) {
    rprintln!("[CONSOLE] USB Serial/JTAG console task started");
    
    // Send welcome banner
    let banner = b"\r\n\r\n\
+==============================================================+\r\n\
|              ESP32-C3 IoT System Console                     |\r\n\
|                    Embassy Framework                         |\r\n\
|                  Direct USB Serial/JTAG                      |\r\n\
+==============================================================+\r\n\
\r\n\
Type 'help' for available commands\r\n\
\r\n\
esp32> ";
    
    match embedded_io_async::Write::write(&mut usb_tx, banner).await {
        Ok(_) => {
            rprintln!("[CONSOLE] Welcome banner sent via USB");
            let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
        }
        Err(e) => rprintln!("[CONSOLE] Banner send error: {:?}", e),
    }
    
    let mut input_buffer = [0u8; 128];
    let mut input_len = 0;
    
    loop {
        let mut byte = [0u8; 1];
        match embedded_io_async::Read::read(&mut usb_rx, &mut byte).await {
            Ok(1) => {
                let ch = byte[0];
                rprintln!("[CONSOLE] Received char: 0x{:02X} ('{}')", ch, ch as char);
                
                match ch {
                    // Enter key - process command
                    b'\r' | b'\n' => {
                        if input_len > 0 {
                            let cmd = core::str::from_utf8(&input_buffer[..input_len]).unwrap_or("");
                            rprintln!("[CONSOLE] Processing command: '{}'", cmd);
                            
                            // Process command and send response
                            let response: &[u8] = match cmd.trim() {
                                "help" | "h" | "?" => {
                                    b"\r\n=== ESP32-C3 IoT Console Commands ===\r\n\
                                    help, h, ?       - Show this help\r\n\
                                    status, stat     - Show system status\r\n\
                                    info, i          - Show system information\r\n\
                                    wifi show        - Show WiFi configuration\r\n\
                                    wifi ssid <name> - Set WiFi SSID\r\n\
                                    wifi pass <pwd>  - Set WiFi password\r\n\
                                    mqtt show        - Show MQTT configuration\r\n\
                                    mqtt broker <ip> - Set MQTT broker\r\n\
                                    clear, cls       - Clear screen\r\n\
                                    restart          - Restart system\r\n\
                                    \r\nesp32> "
                                },
                                "status" | "stat" => {
                                    b"\r\n=== System Status ===\r\n\
                                    WiFi: Not connected\r\n\
                                    MQTT: Not connected\r\n\
                                    Sensor: Not configured\r\n\
                                    System: Online\r\n\
                                    Console: USB Serial/JTAG Active\r\n\
                                    Interface: /dev/ttyACM0\r\n\
                                    \r\nesp32> "
                                },
                                "info" | "i" => {
                                    b"\r\n=== System Information ===\r\n\
                                    Chip: ESP32-C3 RISC-V\r\n\
                                    Framework: Embassy Async\r\n\
                                    Console: Direct USB Serial/JTAG\r\n\
                                    Interface: /dev/ttyACM0\r\n\
                                    Build: Release\r\n\
                                    Heap: Available\r\n\
                                    \r\nesp32> "
                                },
                                "wifi show" => {
                                    b"\r\n=== WiFi Configuration ===\r\n\
                                    SSID: (not configured)\r\n\
                                    Password: (not configured)\r\n\
                                    Status: Disconnected\r\n\
                                    IP: None\r\n\
                                    \r\nesp32> "
                                },
                                "mqtt show" => {
                                    b"\r\n=== MQTT Configuration ===\r\n\
                                    Broker: (not configured)\r\n\
                                    Port: 1883\r\n\
                                    Client ID: esp32-c3\r\n\
                                    Topic Prefix: esp32\r\n\
                                    Status: Disconnected\r\n\
                                    \r\nesp32> "
                                },
                                "clear" | "cls" => {
                                    b"\x1B[2J\x1B[H\r\nesp32> "
                                },
                                "restart" => {
                                    b"\r\nRestarting system...\r\n\
                                    (restart not implemented in demo)\r\n\
                                    \r\nesp32> "
                                },
                                "" => b"\r\nesp32> ",
                                _ => {
                                    if cmd.starts_with("wifi ssid ") {
                                        b"\r\nWiFi SSID setting not implemented in demo\r\nesp32> "
                                    } else if cmd.starts_with("wifi pass ") {
                                        b"\r\nWiFi password setting not implemented in demo\r\nesp32> "
                                    } else if cmd.starts_with("mqtt broker ") {
                                        b"\r\nMQTT broker setting not implemented in demo\r\nesp32> "
                                    } else {
                                        b"\r\nUnknown command. Type 'help' for available commands.\r\nesp32> "
                                    }
                                },
                            };
                            
                            // Send response
                            let _ = embedded_io_async::Write::write(&mut usb_tx, response).await;
                            let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                            rprintln!("[CONSOLE] Response sent");
                        } else {
                            // Just show prompt
                            let _ = embedded_io_async::Write::write(&mut usb_tx, b"\r\nesp32> ").await;
                            let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                        }
                        input_len = 0;
                    },
                    
                    // Backspace
                    0x08 | 0x7F => {
                        if input_len > 0 {
                            input_len -= 1;
                            // Echo backspace
                            let _ = embedded_io_async::Write::write(&mut usb_tx, b"\x08 \x08").await;
                            let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                        }
                    },
                    
                    // Printable characters
                    ch if ch >= 0x20 && ch <= 0x7E => {
                        if input_len < input_buffer.len() - 1 {
                            input_buffer[input_len] = ch;
                            input_len += 1;
                            // Echo character
                            let _ = embedded_io_async::Write::write(&mut usb_tx, &[ch]).await;
                            let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                        }
                    },
                    
                    // Ignore other characters
                    _ => {}
                }
            }
            Ok(_) => {}
            Err(e) => rprintln!("[CONSOLE] Read error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn status_monitor_task() {
    rprintln!("[MONITOR] Status monitor task started");
    
    let mut counter = 0;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
        counter += 1;
        rprintln!("[MONITOR] System heartbeat #{} - Console active", counter);
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 32 * 1024);
    
    // Initialize RTT for debugging
    rtt_init_print!();
    
    rprintln!("[CONSOLE] ESP32-C3 Direct USB Serial/JTAG Console Starting");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("[CONSOLE] Embassy time driver initialized");

    // Configure USB Serial/JTAG peripheral for async
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let usb_serial = usb_serial.into_async();
    let (usb_rx, usb_tx) = usb_serial.split();
    rprintln!("[CONSOLE] USB Serial/JTAG configured");

    // Spawn console tasks
    spawner.spawn(usb_console_task(usb_tx, usb_rx)).ok();
    spawner.spawn(status_monitor_task()).ok();
    
    rprintln!("[CONSOLE] Console task spawned");
    rprintln!("[CONSOLE] ============================================");
    rprintln!("[CONSOLE] Console accessible via /dev/ttyACM0");
    rprintln!("[CONSOLE] Connect with: picocom /dev/ttyACM0 -b 115200");
    rprintln!("[CONSOLE] Commands: help, status, info, wifi, mqtt");
    rprintln!("[CONSOLE] ============================================");
    
    // Main loop
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(30)).await;
        rprintln!("[CONSOLE] Main loop alive - USB console running");
    }
}