//! Serial console implementation using Embassy async framework
//! 
//! Provides an interactive serial console interface for ESP32-C3 system
//! management including configuration and monitoring capabilities.

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use heapless::String;
use rtt_target::rprintln;

use crate::commands::{CommandHandler, MAX_CMD_LEN};
use crate::config::SystemConfig;

/// Maximum input buffer size
#[allow(dead_code)]
const INPUT_BUFFER_SIZE: usize = 128;
/// Command prompt string
const PROMPT: &str = "esp32> ";

/// Serial console manager
pub struct SerialConsole {
    command_handler: Mutex<CriticalSectionRawMutex, CommandHandler>,
    input_buffer: Mutex<CriticalSectionRawMutex, String<MAX_CMD_LEN>>,
}

impl SerialConsole {
    /// Create a new serial console
    pub fn new() -> Self {
        Self {
            command_handler: Mutex::new(CommandHandler::new()),
            input_buffer: Mutex::new(String::new()),
        }
    }
    
    /// Show welcome banner
    pub async fn show_banner<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: Write,
    {
        let banner = "\r\n\
                     ╔══════════════════════════════════════════════════════════════╗\r\n\
                     ║              ESP32-C3 IoT System Console                     ║\r\n\
                     ║                    Embassy Framework                         ║\r\n\
                     ╚══════════════════════════════════════════════════════════════╝\r\n\
                     \r\n\
                     Type 'help' for available commands\r\n\
                     \r\n";
        
        writer.write_all(banner.as_bytes()).await?;
        self.show_prompt(writer).await
    }
    
    /// Show command prompt
    pub async fn show_prompt<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: Write,
    {
        writer.write_all(PROMPT.as_bytes()).await
    }
    
    /// Process a single character input
    pub async fn process_char<W>(&self, writer: &mut W, ch: u8) -> Result<bool, W::Error>
    where
        W: Write,
    {
        let mut input_buffer = self.input_buffer.lock().await;
        
        match ch {
            // Carriage return or line feed - execute command
            b'\r' | b'\n' => {
                writer.write_all(b"\r\n").await?;
                
                let command_line = input_buffer.clone();
                input_buffer.clear();
                
                if !command_line.is_empty() {
                    self.execute_command_line(writer, &command_line).await?;
                }
                
                self.show_prompt(writer).await?;
                Ok(true) // Command processed
            },
            
            // Backspace - remove last character
            0x08 | 0x7F => {
                if !input_buffer.is_empty() {
                    input_buffer.pop();
                    // Send backspace, space, backspace to clear character
                    writer.write_all(b"\x08 \x08").await?;
                }
                Ok(false)
            },
            
            // Escape sequences (arrow keys, etc.) - ignore for now
            0x1B => {
                // Could implement command history here
                Ok(false)
            },
            
            // Printable characters
            ch if ch >= 0x20 && ch <= 0x7E => {
                if input_buffer.len() < MAX_CMD_LEN - 1 {
                    if let Ok(char_ch) = core::str::from_utf8(&[ch]) {
                        if input_buffer.push_str(char_ch).is_ok() {
                            writer.write_all(&[ch]).await?;
                        }
                    }
                }
                Ok(false)
            },
            
            // Control characters - ignore
            _ => Ok(false),
        }
    }
    
    /// Execute a command line
    async fn execute_command_line<W>(&self, writer: &mut W, line: &str) -> Result<(), W::Error>
    where
        W: Write,
    {
        rprintln!("[CONSOLE] Executing command: {}", line);
        
        let mut handler = self.command_handler.lock().await;
        let command = handler.parse_command(line);
        let response = handler.execute_command(command);
        
        if !response.is_empty() {
            writer.write_all(response.as_bytes()).await?;
        }
        
        Ok(())
    }
    
    /// Update system status for display
    pub async fn update_system_status(&self, wifi_connected: bool, mqtt_connected: bool, sensor_active: bool, current_ip: Option<&str>) {
        let mut handler = self.command_handler.lock().await;
        handler.update_system_status(wifi_connected, mqtt_connected, sensor_active, current_ip);
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> SystemConfig {
        let handler = self.command_handler.lock().await;
        handler.get_config().clone()
    }
}

/// UART console task for handling serial input/output
pub async fn uart_console_task<R, W>(console: &SerialConsole, mut reader: R, mut writer: W)
where
    R: Read,
    W: Write,
{
    rprintln!("[CONSOLE] Starting UART console task");
    
    // Show welcome banner
    if let Err(_) = console.show_banner(&mut writer).await {
        rprintln!("[CONSOLE] ERROR: Failed to show banner");
        return;
    }
    
    let mut buffer = [0u8; 1];
    
    loop {
        // Read single character
        match reader.read_exact(&mut buffer).await {
            Ok(()) => {
                let ch = buffer[0];
                
                // Process the character
                match console.process_char(&mut writer, ch).await {
                    Ok(_) => {
                        // Character processed successfully
                    },
                    Err(_) => {
                        rprintln!("[CONSOLE] ERROR: Failed to process character");
                        // Could implement error recovery here
                    }
                }
            },
            Err(_) => {
                // Read error - could be timeout or disconnection
                rprintln!("[CONSOLE] WARNING: UART read error");
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

// RTT console functionality removed - use UART console instead
// This was causing cfg warnings since rtt-console feature doesn't exist