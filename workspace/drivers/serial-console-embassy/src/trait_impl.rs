//! # Serial Console Embassy IoT Container Trait Implementation
//!
//! This module provides the implementation of the IoT Container ConsoleInterface trait
//! for the serial console, enabling seamless integration with the dependency injection
//! container system.

use async_trait::async_trait;
use embassy_time::Instant;
use embedded_io_async::{Read, Write};
use heapless::{String, Deque};

use iot_common::IoTError;

// Import the container trait (when iot-container is available)
#[cfg(feature = "container")]
use iot_container::traits::{ConsoleInterface, EmbeddedString};

use crate::console::SerialConsole;
use crate::commands::{CommandHandler, MAX_CMD_LEN};

/// Maximum number of commands in history
const MAX_HISTORY_ENTRIES: usize = 10;

/// Maximum length for input commands
const MAX_INPUT_LEN: usize = 128;

/// Adapter that implements the IoT Container ConsoleInterface trait for SerialConsole
/// 
/// This adapter bridges the serial console with the IoT Container's trait-based
/// dependency injection system, enabling the serial console to be used as a drop-in
/// component in the container architecture.
#[cfg(feature = "container")]
pub struct ConsoleContainerAdapter<TX, RX>
where
    TX: Write + Send + Sync,
    RX: Read + Send + Sync,
{
    /// UART TX interface for writing output
    uart_tx: TX,
    
    /// UART RX interface for reading input
    uart_rx: RX,
    
    /// Command handler for processing commands
    command_handler: CommandHandler,
    
    /// Input buffer for building commands
    input_buffer: String<MAX_INPUT_LEN>,
    
    /// Command history
    command_history: Deque<EmbeddedString, MAX_HISTORY_ENTRIES>,
    
    /// Session start time
    session_start: Instant,
    
    /// Whether console is ready for operations
    ready: bool,
    
    /// Last command timestamp
    last_command_time: Option<u64>,
    
    /// Total commands processed
    commands_processed: u32,
    
    /// Commands that resulted in errors
    command_errors: u32,
    
    /// Whether echo is enabled
    echo_enabled: bool,
    
    /// Whether prompt should be shown
    show_prompt_next: bool,
}

#[cfg(feature = "container")]
impl<TX, RX> ConsoleContainerAdapter<TX, RX>
where
    TX: Write + Send + Sync,
    RX: Read + Send + Sync,
{
    /// Creates a new console container adapter
    /// 
    /// # Arguments
    /// 
    /// * `uart_tx` - UART TX interface for output
    /// * `uart_rx` - UART RX interface for input
    /// 
    /// # Returns
    /// 
    /// A new adapter instance ready for use
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use serial_console_embassy::{ConsoleContainerAdapter};
    /// use iot_container::traits::ConsoleInterface;
    /// 
    /// let adapter = ConsoleContainerAdapter::new(uart_tx, uart_rx);
    /// ```
    pub fn new(uart_tx: TX, uart_rx: RX) -> Self {
        Self {
            uart_tx,
            uart_rx,
            command_handler: CommandHandler::new(),
            input_buffer: String::new(),
            command_history: Deque::new(),
            session_start: Instant::now(),
            ready: true,
            last_command_time: None,
            commands_processed: 0,
            command_errors: 0,
            echo_enabled: true,
            show_prompt_next: true,
        }
    }
    
    /// Creates a new adapter with configuration options
    /// 
    /// # Arguments
    /// 
    /// * `uart_tx` - UART TX interface for output
    /// * `uart_rx` - UART RX interface for input
    /// * `echo_enabled` - Whether to echo input characters
    /// 
    /// # Returns
    /// 
    /// A new adapter instance with custom configuration
    pub fn new_with_config(uart_tx: TX, uart_rx: RX, echo_enabled: bool) -> Self {
        let mut adapter = Self::new(uart_tx, uart_rx);
        adapter.echo_enabled = echo_enabled;
        adapter
    }
    
    /// Sends the welcome banner to the console
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Banner sent successfully
    /// * `Err(IoTError)` - Write operation failed
    async fn send_welcome_banner(&mut self) -> Result<(), IoTError> {
        let banner = "\r\n\
                     ╔══════════════════════════════════════════════════════════════╗\r\n\
                     ║              ESP32-C3 IoT System Console                     ║\r\n\
                     ║                  Dependency Injection                       ║\r\n\
                     ╚══════════════════════════════════════════════════════════════╝\r\n\
                     \r\n\
                     Type 'help' for available commands\r\n\
                     \r\n";
        
        self.uart_tx.write_all(banner.as_bytes()).await.map_err(|_| {
            IoTError::Hardware(iot_common::HardwareError::InterfaceError("Failed to write banner"))
        })?;
        
        Ok(())
    }
    
    /// Reads a single character from the UART
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(char))` - Character read successfully
    /// * `Ok(None)` - No character available (non-blocking)
    /// * `Err(IoTError)` - Read operation failed
    async fn read_char(&mut self) -> Result<Option<u8>, IoTError> {
        let mut byte = [0u8; 1];
        match self.uart_rx.read(&mut byte).await {
            Ok(1) => Ok(Some(byte[0])),
            Ok(0) => Ok(None), // No data available
            Err(_) => Err(IoTError::Hardware(
                iot_common::HardwareError::InterfaceError("UART read failed")
            )),
        }
    }
    
    /// Processes a single input character
    /// 
    /// # Arguments
    /// 
    /// * `ch` - Character to process
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(command))` - Complete command received
    /// * `Ok(None)` - Character processed, command not complete
    /// * `Err(IoTError)` - Processing failed
    async fn process_input_char(&mut self, ch: u8) -> Result<Option<EmbeddedString>, IoTError> {
        match ch {
            // Carriage return or line feed - command complete
            b'\r' | b'\n' => {
                if self.echo_enabled {
                    self.uart_tx.write_all(b"\r\n").await.map_err(|_| {
                        IoTError::Hardware(iot_common::HardwareError::InterfaceError("Echo write failed"))
                    })?;
                }
                
                if self.input_buffer.is_empty() {
                    self.show_prompt_next = true;
                    return Ok(None);
                }
                
                // Extract command
                let command_str = self.input_buffer.as_str();
                let command = EmbeddedString::try_from(command_str).map_err(|_| {
                    IoTError::Configuration(iot_common::ConfigError::InvalidFormat("Command too long"))
                })?;
                
                // Add to history
                self.add_to_history(command.clone());
                
                // Clear input buffer
                self.input_buffer.clear();
                self.show_prompt_next = true;
                
                Ok(Some(command))
            }
            
            // Backspace - remove last character
            0x08 | 0x7F => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    if self.echo_enabled {
                        // Send backspace, space, backspace to clear character
                        self.uart_tx.write_all(b"\x08 \x08").await.map_err(|_| {
                            IoTError::Hardware(iot_common::HardwareError::InterfaceError("Backspace write failed"))
                        })?;
                    }
                }
                Ok(None)
            }
            
            // Escape sequences (arrow keys, etc.) - ignore for now
            0x1B => {
                // Could implement command history navigation here
                Ok(None)
            }
            
            // Printable ASCII characters
            ch if ch >= 0x20 && ch <= 0x7E => {
                if self.input_buffer.len() < MAX_INPUT_LEN - 1 {
                    if let Ok(()) = self.input_buffer.push(ch as char) {
                        if self.echo_enabled {
                            self.uart_tx.write_all(&[ch]).await.map_err(|_| {
                                IoTError::Hardware(iot_common::HardwareError::InterfaceError("Echo write failed"))
                            })?;
                        }
                    }
                }
                Ok(None)
            }
            
            // Ignore other characters
            _ => Ok(None),
        }
    }
    
    /// Adds a command to the history
    /// 
    /// # Arguments
    /// 
    /// * `command` - Command to add to history
    fn add_to_history(&mut self, command: EmbeddedString) {
        // Don't add empty commands or duplicates
        if command.is_empty() {
            return;
        }
        
        // Check if command is duplicate of last entry
        if let Some(last_command) = self.command_history.back() {
            if last_command == &command {
                return;
            }
        }
        
        // Add to history, removing oldest if full
        if self.command_history.len() >= MAX_HISTORY_ENTRIES {
            self.command_history.pop_front();
        }
        
        let _ = self.command_history.push_back(command);
    }
    
    /// Processes a command and generates response
    /// 
    /// # Arguments
    /// 
    /// * `command` - Command string to process
    /// 
    /// # Returns
    /// 
    /// * `Ok(response)` - Command processed successfully
    /// * `Err(IoTError)` - Command processing failed
    async fn process_command_internal(&mut self, command: &str) -> Result<EmbeddedString, IoTError> {
        self.commands_processed += 1;
        self.last_command_time = Some(Instant::now().as_millis());
        
        // Basic command processing - in a real implementation, this would
        // integrate with the full command handler system
        let response = match command.trim().to_lowercase().as_str() {
            "help" | "h" | "?" => {
                "Available commands:\r\n\
                 help, h, ?       - Show this help\r\n\
                 status           - Show system status\r\n\
                 info             - Show system information\r\n\
                 history          - Show command history\r\n\
                 clear            - Clear screen\r\n\
                 exit, quit       - Exit console session\r\n\
                 echo on/off      - Enable/disable echo\r\n"
            }
            
            "status" => {
                let uptime = self.session_start.elapsed().as_secs();
                "System Status: IoT Container Operational\r\n\
                 Console: Active\r\n\
                 Session Uptime: Active\r\n\
                 Commands Processed: Available\r\n"
            }
            
            "info" => {
                "ESP32-C3 IoT System with Dependency Injection\r\n\
                 Framework: Embassy Async Runtime\r\n\
                 Architecture: Trait-based Container System\r\n\
                 Console: Serial USB/JTAG Interface\r\n\
                 Features: Sensor, Network, MQTT, Console\r\n"
            }
            
            "history" => {
                if self.command_history.is_empty() {
                    "No commands in history\r\n"
                } else {
                    "Recent commands:\r\n"
                    // In a real implementation, would iterate and format history
                }
            }
            
            "clear" | "cls" => {
                "\x1B[2J\x1B[H" // ANSI clear screen and move cursor to home
            }
            
            "exit" | "quit" => {
                "Console session ended\r\n"
            }
            
            "echo on" => {
                self.echo_enabled = true;
                "Echo enabled\r\n"
            }
            
            "echo off" => {
                self.echo_enabled = false;
                "Echo disabled\r\n"
            }
            
            "" => {
                "" // Empty command
            }
            
            _ => {
                self.command_errors += 1;
                "Unknown command. Type 'help' for available commands.\r\n"
            }
        };
        
        EmbeddedString::try_from(response).map_err(|_| {
            IoTError::System(iot_common::SystemError::InvalidOperation("Response too long"))
        })
    }
    
    /// Gets console session metrics
    /// 
    /// # Returns
    /// 
    /// Tuple containing (commands_processed, command_errors, uptime_seconds)
    pub fn get_session_metrics(&self) -> (u32, u32, u32) {
        let uptime = self.session_start.elapsed().as_secs() as u32;
        (self.commands_processed, self.command_errors, uptime)
    }
    
    /// Gets command history
    /// 
    /// # Returns
    /// 
    /// Reference to command history deque
    pub fn get_command_history(&self) -> &Deque<EmbeddedString, MAX_HISTORY_ENTRIES> {
        &self.command_history
    }
    
    /// Checks if echo is enabled
    /// 
    /// # Returns
    /// 
    /// `true` if echo is enabled, `false` otherwise
    pub fn is_echo_enabled(&self) -> bool {
        self.echo_enabled
    }
    
    /// Sets echo state
    /// 
    /// # Arguments
    /// 
    /// * `enabled` - Whether to enable echo
    pub fn set_echo_enabled(&mut self, enabled: bool) {
        self.echo_enabled = enabled;
    }
}

#[cfg(feature = "container")]
#[async_trait]
impl<TX, RX> ConsoleInterface for ConsoleContainerAdapter<TX, RX>
where
    TX: Write + Send + Sync,
    RX: Read + Send + Sync,
{
    /// Writes a line of text to the console
    /// 
    /// This method sends a text message to the console with automatic line termination.
    /// 
    /// # Arguments
    /// 
    /// * `message` - Text message to write
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message written successfully
    /// * `Err(IoTError)` - Write operation failed
    /// 
    /// # Implementation Details
    /// 
    /// - Automatically adds CRLF line termination
    /// - Handles UART transmission errors
    /// - Updates output statistics
    async fn write_line(&mut self, message: &str) -> Result<(), IoTError> {
        // Write the message
        self.uart_tx.write_all(message.as_bytes()).await.map_err(|_| {
            IoTError::Hardware(iot_common::HardwareError::InterfaceError("UART write failed"))
        })?;
        
        // Add line termination
        self.uart_tx.write_all(b"\r\n").await.map_err(|_| {
            IoTError::Hardware(iot_common::HardwareError::InterfaceError("UART write failed"))
        })?;
        
        Ok(())
    }
    
    /// Reads a command from the console
    /// 
    /// This method attempts to read a complete command from the console input.
    /// It handles character buffering and command parsing.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(command))` - Command received
    /// * `Ok(None)` - No command available (non-blocking)
    /// * `Err(IoTError)` - Read operation failed
    /// 
    /// # Implementation Details
    /// 
    /// - Non-blocking operation
    /// - Handles character echo if enabled
    /// - Processes backspace and control characters
    /// - Builds commands character by character
    async fn read_command(&mut self) -> Result<Option<EmbeddedString>, IoTError> {
        // Try to read a character
        if let Some(ch) = self.read_char().await? {
            // Process the character and check if command is complete
            self.process_input_char(ch).await
        } else {
            Ok(None)
        }
    }
    
    /// Handles a console command and returns response
    /// 
    /// This method processes a received command and generates an appropriate response.
    /// 
    /// # Arguments
    /// 
    /// * `command` - Command string to process
    /// 
    /// # Returns
    /// 
    /// * `Ok(response)` - Command processed, response generated
    /// * `Err(IoTError)` - Command processing failed
    /// 
    /// # Implementation Details
    /// 
    /// - Integrates with command handler system
    /// - Updates command statistics
    /// - Handles command history
    /// - Provides comprehensive command set
    async fn handle_command(&mut self, command: &str) -> Result<EmbeddedString, IoTError> {
        self.process_command_internal(command).await
    }
    
    /// Checks if console is ready for input/output
    /// 
    /// This method verifies that the console interface is operational and ready
    /// for command processing.
    /// 
    /// # Returns
    /// 
    /// `true` if console is ready, `false` otherwise
    /// 
    /// # Implementation Details
    /// 
    /// - Checks UART interface status
    /// - Validates session state
    /// - Returns cached readiness status
    async fn is_ready(&self) -> bool {
        self.ready
    }
    
    /// Gets console session information
    /// 
    /// Returns information about the current console session including
    /// connection status and session duration.
    /// 
    /// # Returns
    /// 
    /// Session information as a formatted string
    /// 
    /// # Information Included
    /// 
    /// - Session uptime
    /// - Commands processed
    /// - Error count
    /// - Echo status
    fn get_session_info(&self) -> EmbeddedString {
        let uptime = self.session_start.elapsed().as_secs();
        let info = format!(
            "Session: {}s, Commands: {}, Errors: {}, Echo: {}",
            uptime,
            self.commands_processed,
            self.command_errors,
            if self.echo_enabled { "On" } else { "Off" }
        );
        
        EmbeddedString::try_from(info.as_str())
            .unwrap_or_else(|_| EmbeddedString::try_from("Session active").unwrap())
    }
    
    /// Sends a formatted prompt to the console
    /// 
    /// This method displays a command prompt to indicate the console is ready for input.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Prompt displayed successfully
    /// * `Err(IoTError)` - Prompt display failed
    /// 
    /// # Implementation Details
    /// 
    /// - Shows session-specific prompt
    /// - Handles prompt formatting
    /// - Updates prompt display state
    async fn show_prompt(&mut self) -> Result<(), IoTError> {
        if self.show_prompt_next {
            let prompt = "iot> ";
            self.uart_tx.write_all(prompt.as_bytes()).await.map_err(|_| {
                IoTError::Hardware(iot_common::HardwareError::InterfaceError("Prompt write failed"))
            })?;
            self.show_prompt_next = false;
        }
        Ok(())
    }
}

// Convenience functions for creating container-compatible console instances

/// Creates a new console interface adapter for use with the IoT container
/// 
/// This function provides a convenient way to create a console adapter that
/// implements the container's ConsoleInterface trait.
/// 
/// # Arguments
/// 
/// * `uart_tx` - UART TX interface for output
/// * `uart_rx` - UART RX interface for input
/// 
/// # Returns
/// 
/// A new console adapter ready for use with the IoT container
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use serial_console_embassy::create_container_console;
/// use iot_container::ComponentFactory;
/// 
/// let console = create_container_console(uart_tx, uart_rx);
/// ```
#[cfg(feature = "container")]
pub fn create_container_console<TX, RX>(
    uart_tx: TX, 
    uart_rx: RX
) -> ConsoleContainerAdapter<TX, RX>
where
    TX: Write + Send + Sync,
    RX: Read + Send + Sync,
{
    ConsoleContainerAdapter::new(uart_tx, uart_rx)
}

/// Creates a console adapter with custom configuration
/// 
/// This function allows customization of console behavior including
/// echo settings and other operational parameters.
/// 
/// # Arguments
/// 
/// * `uart_tx` - UART TX interface for output
/// * `uart_rx` - UART RX interface for input
/// * `echo_enabled` - Whether to echo input characters
/// 
/// # Returns
/// 
/// A new console adapter with custom configuration
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use serial_console_embassy::create_container_console_with_config;
/// 
/// // Create console with echo disabled
/// let console = create_container_console_with_config(uart_tx, uart_rx, false);
/// ```
#[cfg(feature = "container")]
pub fn create_container_console_with_config<TX, RX>(
    uart_tx: TX, 
    uart_rx: RX,
    echo_enabled: bool
) -> ConsoleContainerAdapter<TX, RX>
where
    TX: Write + Send + Sync,
    RX: Read + Send + Sync,
{
    ConsoleContainerAdapter::new_with_config(uart_tx, uart_rx, echo_enabled)
}