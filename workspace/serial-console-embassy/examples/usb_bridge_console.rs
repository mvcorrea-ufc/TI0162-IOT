//! USB Serial/JTAG Bridge Console for ESP32-C3
//! 
//! Bridges UART0 console with USB Serial/JTAG peripheral
//! Console accessible via /dev/ttyACM0 with no external hardware

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::pipe::Pipe;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::{
    uart::{Uart, UartRx, UartTx, Config as UartConfig},
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
    timer::timg::TimerGroup,
    gpio::Io,
    Async,
};
use embedded_io_async::{Read, Write};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

const BRIDGE_BUF_SIZE: usize = 256;

// Bidirectional bridge pipes: UART ↔ USB Serial/JTAG
static UART_TO_USB_PIPE: Pipe<CriticalSectionRawMutex, BRIDGE_BUF_SIZE> = Pipe::new();
static USB_TO_UART_PIPE: Pipe<CriticalSectionRawMutex, BRIDGE_BUF_SIZE> = Pipe::new();

#[embassy_executor::task]
async fn uart_to_usb_bridge(mut uart_rx: UartRx<'static, Async>) {
    rprintln!("[BRIDGE] UART→USB bridge task started");
    let mut buffer = [0u8; 64];
    
    loop {
        match embedded_io_async::Read::read(&mut uart_rx, &mut buffer).await {
            Ok(len) if len > 0 => {
                rprintln!("[BRIDGE] UART→USB: {} bytes", len);
                UART_TO_USB_PIPE.write_all(&buffer[..len]).await;
            }
            Ok(_) => {}
            Err(e) => rprintln!("[BRIDGE] UART read error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn usb_to_uart_bridge(mut uart_tx: UartTx<'static, Async>) {
    rprintln!("[BRIDGE] USB→UART bridge task started");
    let mut buffer = [0u8; 64];
    
    loop {
        USB_TO_UART_PIPE.read(&mut buffer).await;
        match embedded_io_async::Write::write(&mut uart_tx, &buffer).await {
            Ok(_) => {
                rprintln!("[BRIDGE] USB→UART: forwarded data");
                let _ = embedded_io_async::Write::flush(&mut uart_tx).await;
            }
            Err(e) => rprintln!("[BRIDGE] UART write error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn usb_reader(mut usb_rx: UsbSerialJtagRx<'static, Async>) {
    rprintln!("[BRIDGE] USB reader task started");
    let mut buffer = [0u8; 64];
    
    loop {
        match embedded_io_async::Read::read(&mut usb_rx, &mut buffer).await {
            Ok(len) if len > 0 => {
                rprintln!("[BRIDGE] USB→UART: {} bytes received", len);
                USB_TO_UART_PIPE.write_all(&buffer[..len]).await;
            }
            Ok(_) => {}
            Err(e) => rprintln!("[BRIDGE] USB read error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn usb_writer(mut usb_tx: UsbSerialJtagTx<'static, Async>) {
    rprintln!("[BRIDGE] USB writer task started");
    let mut buffer = [0u8; 64];
    
    // Send welcome banner via USB Serial/JTAG
    let banner = b"\r\n\r\n\
╔══════════════════════════════════════════════════════════════╗\r\n\
║              ESP32-C3 IoT System Console                     ║\r\n\
║                    Embassy Framework                         ║\r\n\
║                  USB Serial/JTAG Bridge                      ║\r\n\
╚══════════════════════════════════════════════════════════════╝\r\n\
\r\n\
Type 'help' for available commands\r\n\
\r\n\
esp32> ";
    
    match embedded_io_async::Write::write(&mut usb_tx, banner).await {
        Ok(_) => {
            rprintln!("[BRIDGE] Welcome banner sent via USB");
            let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
        }
        Err(e) => rprintln!("[BRIDGE] Banner send error: {:?}", e),
    }
    
    loop {
        // Forward UART output to USB Serial/JTAG
        UART_TO_USB_PIPE.read(&mut buffer).await;
        match embedded_io_async::Write::write(&mut usb_tx, &buffer).await {
            Ok(_) => {
                rprintln!("[BRIDGE] UART→USB: forwarded data");
                let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
            }
            Err(e) => rprintln!("[BRIDGE] USB write error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn uart_console_task(mut uart_tx: UartTx<'static, Async>, mut uart_rx: UartRx<'static, Async>) {
    rprintln!("[CONSOLE] UART console task started");
    
    let mut input_buffer = [0u8; 128];
    let mut input_len = 0;
    
    loop {
        let mut byte = [0u8; 1];
        match embedded_io_async::Read::read(&mut uart_rx, &mut byte).await {
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
                                    mqtt show        - Show MQTT configuration\r\n\
                                    clear, cls       - Clear screen\r\n\
                                    \r\nesp32> "
                                },
                                "status" | "stat" => {
                                    b"\r\n=== System Status ===\r\n\
                                    WiFi: Not connected\r\n\
                                    MQTT: Not connected\r\n\
                                    Sensor: Not configured\r\n\
                                    System: Online\r\n\
                                    Bridge: USB Serial/JTAG Active\r\n\
                                    \r\nesp32> "
                                },
                                "info" | "i" => {
                                    b"\r\n=== System Information ===\r\n\
                                    Chip: ESP32-C3\r\n\
                                    Framework: Embassy\r\n\
                                    Console: USB Serial/JTAG Bridge\r\n\
                                    Interface: /dev/ttyACM0\r\n\
                                    Build: Release\r\n\
                                    \r\nesp32> "
                                },
                                "wifi show" => {
                                    b"\r\n=== WiFi Configuration ===\r\n\
                                    SSID: (not configured)\r\n\
                                    Password: (not configured)\r\n\
                                    Status: Disconnected\r\n\
                                    \r\nesp32> "
                                },
                                "mqtt show" => {
                                    b"\r\n=== MQTT Configuration ===\r\n\
                                    Broker: (not configured)\r\n\
                                    Port: 1883\r\n\
                                    Client ID: esp32-c3\r\n\
                                    Status: Disconnected\r\n\
                                    \r\nesp32> "
                                },
                                "clear" | "cls" => {
                                    b"\x1B[2J\x1B[H\r\nesp32> "
                                },
                                "" => b"\r\nesp32> ",
                                _ => b"\r\nUnknown command. Type 'help' for available commands.\r\nesp32> ",
                            };
                            
                            // Send response
                            let _ = embedded_io_async::Write::write(&mut uart_tx, response).await;
                            let _ = embedded_io_async::Write::flush(&mut uart_tx).await;
                            rprintln!("[CONSOLE] Response sent");
                        } else {
                            // Just show prompt
                            let _ = embedded_io_async::Write::write(&mut uart_tx, b"\r\nesp32> ").await;
                            let _ = embedded_io_async::Write::flush(&mut uart_tx).await;
                        }
                        input_len = 0;
                    },
                    
                    // Backspace
                    0x08 | 0x7F => {
                        if input_len > 0 {
                            input_len -= 1;
                            // Echo backspace
                            let _ = embedded_io_async::Write::write(&mut uart_tx, b"\x08 \x08").await;
                            let _ = embedded_io_async::Write::flush(&mut uart_tx).await;
                        }
                    },
                    
                    // Printable characters
                    ch if ch >= 0x20 && ch <= 0x7E => {
                        if input_len < input_buffer.len() - 1 {
                            input_buffer[input_len] = ch;
                            input_len += 1;
                            // Echo character
                            let _ = embedded_io_async::Write::write(&mut uart_tx, &[ch]).await;
                            let _ = embedded_io_async::Write::flush(&mut uart_tx).await;
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

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 32 * 1024);
    
    // Initialize RTT for debugging
    rtt_init_print!();
    
    rprintln!("[BRIDGE] ESP32-C3 USB Serial/JTAG Bridge Console Starting");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("[BRIDGE] Embassy time driver initialized");

    // Configure USB Serial/JTAG peripheral
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let (usb_rx, usb_tx) = usb_serial.split();
    let (usb_rx, usb_tx) = (usb_rx.into_async(), usb_tx.into_async());
    rprintln!("[BRIDGE] USB Serial/JTAG configured");

    // Configure UART0 for internal console
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let tx_pin = io.pins.gpio21;  // WeAct ESP32-C3 UART0 TX
    let rx_pin = io.pins.gpio20;  // WeAct ESP32-C3 UART0 RX
    
    let uart_config = UartConfig::default();
    let uart = Uart::new_with_config(peripherals.UART0, uart_config, Some(tx_pin), Some(rx_pin), None, None).unwrap();
    let (uart_tx, uart_rx) = uart.split();
    let (uart_tx, uart_rx) = (uart_tx.into_async(), uart_rx.into_async());
    rprintln!("[BRIDGE] UART0 configured on GPIO20/21");

    // Spawn bridge tasks
    spawner.spawn(uart_to_usb_bridge(uart_rx)).ok();
    spawner.spawn(usb_to_uart_bridge(uart_tx)).ok();
    spawner.spawn(usb_reader(usb_rx)).ok();
    spawner.spawn(usb_writer(usb_tx)).ok();
    
    // Note: We can't use uart_console_task here because we've already used uart_tx/uart_rx
    // The bridge itself handles the console functionality
    
    rprintln!("[BRIDGE] All bridge tasks spawned");
    rprintln!("[BRIDGE] Console accessible via /dev/ttyACM0");
    rprintln!("[BRIDGE] Connect with: picocom /dev/ttyACM0 -b 115200");
    
    // Main loop
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(30)).await;
        rprintln!("[BRIDGE] System alive - USB bridge running");
    }
}