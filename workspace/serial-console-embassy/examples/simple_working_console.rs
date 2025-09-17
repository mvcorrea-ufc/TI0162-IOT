//! Simple Working Console for ESP32-C3
//! 
//! Uses default UART0 configuration to test if /dev/ttyACM0 is connected

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    uart::{Uart, UartRx, UartTx, Config as UartConfig},
    timer::timg::TimerGroup,
    Async,
};
use embedded_io_async::{Read, Write};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[embassy_executor::task]
async fn console_task(mut tx: UartTx<'static, Async>, mut rx: UartRx<'static, Async>) {
    rprintln!("[CONSOLE] Console task started - sending banner");
    
    // Send banner immediately
    let banner = b"\r\n\r\n*** ESP32-C3 Embassy Console ***\r\n\r\nType 'hello' and press Enter:\r\n> ";
    
    match embedded_io_async::Write::write(&mut tx, banner).await {
        Ok(_) => {
            rprintln!("[CONSOLE] Banner sent successfully!");
            let _ = embedded_io_async::Write::flush(&mut tx).await;
        }
        Err(e) => rprintln!("[CONSOLE] Banner error: {:?}", e),
    }
    
    let mut buffer = [0u8; 1];
    let mut input = [0u8; 64];
    let mut input_len = 0;
    
    loop {
        match embedded_io_async::Read::read(&mut rx, &mut buffer).await {
            Ok(1) => {
                let ch = buffer[0];
                rprintln!("[CONSOLE] Received: 0x{:02X} '{}'", ch, ch as char);
                
                if ch == b'\r' || ch == b'\n' {
                    // Process input
                    if input_len > 0 {
                        let cmd = core::str::from_utf8(&input[..input_len]).unwrap_or("");
                        rprintln!("[CONSOLE] Command: '{}'", cmd);
                        
                        let response: &[u8] = if cmd == "hello" {
                            b"\r\nHello from ESP32-C3!\r\n> "
                        } else {
                            b"\r\nUnknown command. Try 'hello'\r\n> "
                        };
                        
                        let _ = embedded_io_async::Write::write(&mut tx, response).await;
                        let _ = embedded_io_async::Write::flush(&mut tx).await;
                        rprintln!("[CONSOLE] Response sent");
                    } else {
                        let _ = embedded_io_async::Write::write(&mut tx, b"\r\n> ").await;
                        let _ = embedded_io_async::Write::flush(&mut tx).await;
                    }
                    input_len = 0;
                } else if ch >= 0x20 && ch <= 0x7E && input_len < 63 {
                    // Printable character
                    input[input_len] = ch;
                    input_len += 1;
                    
                    // Echo character
                    let _ = embedded_io_async::Write::write(&mut tx, &[ch]).await;
                    let _ = embedded_io_async::Write::flush(&mut tx).await;
                } else if ch == 0x08 || ch == 0x7F {
                    // Backspace
                    if input_len > 0 {
                        input_len -= 1;
                        let _ = embedded_io_async::Write::write(&mut tx, b"\x08 \x08").await;
                        let _ = embedded_io_async::Write::flush(&mut tx).await;
                    }
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
    
    rprintln!("[CONSOLE] ESP32-C3 Simple Console Test");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("[CONSOLE] Embassy initialized");

    // Try default UART0 first (might be connected to USB-C on WeAct)
    let uart_config = UartConfig::default();
    let uart = Uart::new(peripherals.UART0, uart_config).unwrap();
    let (uart_tx, uart_rx) = uart.split();
    let (tx, rx) = (uart_tx.into_async(), uart_rx.into_async());

    rprintln!("[CONSOLE] UART0 configured with default pins");
    rprintln!("[CONSOLE] Testing if /dev/ttyACM0 receives console output...");

    // Spawn console task
    spawner.spawn(console_task(rx, tx)).ok();

    rprintln!("[CONSOLE] Console task spawned");
    rprintln!("[CONSOLE] Check picocom /dev/ttyACM0 for banner!");
    
    // Send periodic test messages
    let mut counter = 0;
    loop {
        Timer::after(Duration::from_secs(10)).await;
        counter += 1;
        rprintln!("[CONSOLE] Test #{} - Check if banner appeared in picocom", counter);
    }
}