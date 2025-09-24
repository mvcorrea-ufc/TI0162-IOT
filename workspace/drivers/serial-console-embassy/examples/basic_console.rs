//! Basic Serial Console Example
//! 
//! Demonstrates a simple serial console interface for ESP32-C3
//! with basic command handling and system information display.
//! 
//! ## Usage
//! 1. Flash to ESP32-C3: cargo run --example basic_console --release
//! 2. Connect via serial terminal at 115200 baud
//! 3. Type 'help' to see available commands

#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    uart::{Uart, UartRx, UartTx, Config as UartConfig},
    timer::timg::TimerGroup,
    Async,
};

use serial_console_embassy::{SerialConsole, console::uart_console_task};

#[embassy_executor::task]
async fn console_task(console: &'static SerialConsole, rx: UartRx<'static, Async>, tx: UartTx<'static, Async>) {
    uart_console_task(console, rx, tx).await;
}

#[embassy_executor::task]
async fn system_monitor_task(console: &'static SerialConsole) {
    rprintln!("[CONSOLE] Starting system monitor task");
    
    let mut counter = 0;
    loop {
        Timer::after(Duration::from_secs(5)).await;
        counter += 1;
        
        // Simulate system status updates
        let wifi_connected = counter % 3 == 0;
        let mqtt_connected = counter % 4 == 0;
        let sensor_active = true;
        let current_ip = if wifi_connected { 
            Some("192.168.1.100") 
        } else { 
            None 
        };
        
        console.update_system_status(
            wifi_connected, 
            mqtt_connected, 
            sensor_active, 
            current_ip
        ).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 32 * 1024);
    
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("[CONSOLE] ESP32-C3 Basic Serial Console Example");
    rprintln!("[CONSOLE] ==========================================");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("[CONSOLE] Embassy time driver initialized");

    // Configure UART for serial console (115200 baud)
    let uart_config = UartConfig::default();
    let uart = Uart::new(peripherals.UART0, uart_config).unwrap();
    let (uart_tx, uart_rx) = uart.split();
    let (tx, rx) = (uart_tx.into_async(), uart_rx.into_async());

    rprintln!("[CONSOLE] UART configured at 115200 baud");

    // Create console instance
    use static_cell::StaticCell;
    static CONSOLE: StaticCell<SerialConsole> = StaticCell::new();
    let console = CONSOLE.init(SerialConsole::new());

    rprintln!("[CONSOLE] Serial console created");

    // Spawn console and monitoring tasks
    spawner.spawn(console_task(console, tx, rx)).ok();
    spawner.spawn(system_monitor_task(console)).ok();

    rprintln!("[CONSOLE] Console tasks spawned");
    rprintln!("[CONSOLE] Connect via serial terminal at 115200 baud");
    rprintln!("[CONSOLE] Type 'help' for available commands");
    
    // Main loop
    loop {
        Timer::after(Duration::from_secs(30)).await;
        rprintln!("[CONSOLE] System running... Connect via UART for interactive console");
    }
}