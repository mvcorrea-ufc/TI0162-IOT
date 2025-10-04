//! HTTP Client Module - Simple HTTP networking test
//! 
//! Provides basic HTTP client functionality to test robust networking
//! before implementing more complex protocols like MQTT.
//!
//! ## Usage:
//! 
//! ### With Blocking WiFi:
//! ```bash
//! cargo run --release --features blocking,http
//! ```
//! 
//! ## Test Strategy:
//! - Test external HTTP service (httpbin.org) for internet connectivity
//! - Test local HTTP services on broker machine for LAN connectivity
//! - Validate HTTP response parsing and error handling

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use rtt_target::rprintln;
use smoltcp::wire::IpAddress;
use embedded_io::{Read, Write};

/// HTTP client configuration
pub struct HttpConfig {
    pub user_agent: &'static str,
    pub timeout_ms: u32,
    pub max_response_size: usize,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            user_agent: "ESP32-C3-Client/1.0",
            timeout_ms: 10000,
            max_response_size: 2048,
        }
    }
}

/// HTTP request structure
pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub host: &'a str,
    pub headers: Vec<(&'a str, &'a str)>,
}

impl<'a> HttpRequest<'a> {
    pub fn new(method: &'a str, host: &'a str, path: &'a str) -> Self {
        Self {
            method,
            path,
            host,
            headers: Vec::new(),
        }
    }
    
    pub fn add_header(mut self, name: &'a str, value: &'a str) -> Self {
        self.headers.push((name, value));
        self
    }
    
    /// Build HTTP request string
    pub fn build_request(&self, config: &HttpConfig) -> String {
        let mut request = String::new();
        
        // Request line
        request.push_str(&format!("{} {} HTTP/1.1\r\n", self.method, self.path));
        
        // Host header (required for HTTP/1.1)
        request.push_str(&format!("Host: {}\r\n", self.host));
        
        // User-Agent
        request.push_str(&format!("User-Agent: {}\r\n", config.user_agent));
        
        // Connection close for simplicity
        request.push_str("Connection: close\r\n");
        
        // Additional headers
        for (name, value) in &self.headers {
            request.push_str(&format!("{}: {}\r\n", name, value));
        }
        
        // End headers
        request.push_str("\r\n");
        
        request
    }
}

/// HTTP response structure
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Parse HTTP response from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        let response_str = core::str::from_utf8(data).map_err(|_| "Invalid UTF-8 response")?;
        
        // Find header/body separator
        let header_end = response_str.find("\r\n\r\n").ok_or("Invalid HTTP response format")?;
        let headers_part = &response_str[..header_end];
        let body_start = header_end + 4;
        
        // Parse status line
        let mut lines = headers_part.lines();
        let status_line = lines.next().ok_or("Missing status line")?;
        
        let status_parts: Vec<&str> = status_line.split_whitespace().collect();
        if status_parts.len() < 3 {
            return Err("Invalid status line format");
        }
        
        let status_code = status_parts[1].parse::<u16>().map_err(|_| "Invalid status code")?;
        let status_text = status_parts[2..].join(" ");
        
        // Parse headers
        let mut headers = Vec::new();
        for line in lines {
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();
                headers.push((name.to_string(), value.to_string()));
            }
        }
        
        // Extract body
        let body = if body_start < data.len() {
            data[body_start..].to_vec()
        } else {
            Vec::new()
        };
        
        Ok(HttpResponse {
            status_code,
            status_text,
            headers,
            body,
        })
    }
    
    /// Get header value by name
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
    
    /// Get body as string
    pub fn body_as_string(&self) -> Result<String, &'static str> {
        String::from_utf8(self.body.clone()).map_err(|_| "Body is not valid UTF-8")
    }
}

/// Simple HTTP client for testing network connectivity
pub struct SimpleHttpClient {
    config: HttpConfig,
}

impl SimpleHttpClient {
    pub fn new(config: HttpConfig) -> Self {
        Self { config }
    }
    
    /// Test HTTP connectivity with external service
    pub fn test_external_connectivity<DeviceT>(
        &self,
        stack: &mut blocking_network_stack::Stack<DeviceT>,
    ) -> Result<(), &'static str>
    where
        DeviceT: smoltcp::phy::Device + 'static,
    {
        rprintln!("HTTP: Testing external connectivity with httpbin.org");
        
        // Create request
        let request = HttpRequest::new("GET", "httpbin.org", "/get")
            .add_header("Accept", "application/json");
        
        let request_str = request.build_request(&self.config);
        rprintln!("HTTP: Request prepared ({} bytes)", request_str.len());
        
        // Create socket with larger buffer for HTTP
        static mut HTTP_RX_BUFFER: [u8; 2048] = [0u8; 2048];
        static mut HTTP_TX_BUFFER: [u8; 1024] = [0u8; 1024];
        let (rx_buffer, tx_buffer) = unsafe { (&mut HTTP_RX_BUFFER, &mut HTTP_TX_BUFFER) };
        let mut socket = stack.get_socket(rx_buffer, tx_buffer);
        
        // Connect to httpbin.org (52.45.110.183) with timeout handling
        let server_addr = IpAddress::v4(52, 45, 110, 183);
        rprintln!("HTTP: Attempting connection to httpbin.org...");
        
        match socket.open(server_addr, 80) {
            Ok(()) => {
                rprintln!("HTTP: ✅ CONNECTION SUCCESSFUL to httpbin.org!");
            }
            Err(e) => {
                rprintln!("HTTP: ❌ Connection failed to httpbin.org: {:?}", e);
                rprintln!("HTTP: This is expected if internet access is limited");
                return Ok(()); // Continue with local tests
            }
        }
        
        rprintln!("HTTP: Connected to httpbin.org");
        
        // Send request
        socket.write_all(request_str.as_bytes()).map_err(|_| "Failed to send request")?;
        socket.flush().map_err(|_| "Failed to flush request")?;
        
        rprintln!("HTTP: Request sent, reading response...");
        
        // Read response
        let mut response_buffer = [0u8; 2048];
        let response_len = socket.read(&mut response_buffer).map_err(|_| "Failed to read response")?;
        
        rprintln!("HTTP: Received {} bytes", response_len);
        
        // Parse response
        match HttpResponse::parse(&response_buffer[..response_len]) {
            Ok(response) => {
                rprintln!("HTTP: ✅ Status: {} {}", response.status_code, response.status_text);
                rprintln!("HTTP: Headers: {} found", response.headers.len());
                
                if response.status_code == 200 {
                    rprintln!("HTTP: ✅ External connectivity test PASSED");
                } else {
                    rprintln!("HTTP: ⚠️ External connectivity test got non-200 status");
                }
            }
            Err(e) => {
                rprintln!("HTTP: ❌ Failed to parse response: {}", e);
                // Show raw response for debugging
                if let Ok(raw) = core::str::from_utf8(&response_buffer[..response_len.min(200)]) {
                    rprintln!("HTTP: Raw response preview: {}", raw);
                }
            }
        }
        
        socket.disconnect();
        Ok(())
    }
    
    /// Test HTTP connectivity with local services
    pub fn test_local_connectivity<DeviceT>(
        &self,
        stack: &mut blocking_network_stack::Stack<DeviceT>,
        local_ip: [u8; 4],
    ) -> Result<(), &'static str>
    where
        DeviceT: smoltcp::phy::Device + 'static,
    {
        rprintln!("HTTP: Testing local connectivity to {}.{}.{}.{}", 
            local_ip[0], local_ip[1], local_ip[2], local_ip[3]);
        
        let test_ports = [80, 8000, 8080, 3000];
        let local_addr = IpAddress::v4(local_ip[0], local_ip[1], local_ip[2], local_ip[3]);
        
        for port in test_ports.iter() {
            rprintln!("HTTP: Testing port {}", port);
            
            // Create socket for each port test
            static mut LOCAL_RX_BUFFER: [u8; 1024] = [0u8; 1024];
            static mut LOCAL_TX_BUFFER: [u8; 512] = [0u8; 512];
            let (rx_buffer, tx_buffer) = unsafe { (&mut LOCAL_RX_BUFFER, &mut LOCAL_TX_BUFFER) };
            let mut socket = stack.get_socket(rx_buffer, tx_buffer);
            
            match socket.open(local_addr, *port) {
                Ok(()) => {
                    rprintln!("HTTP: Connected to port {}", port);
                    
                    // Create simple request
                    let host_header = format!("{}.{}.{}.{}", local_ip[0], local_ip[1], local_ip[2], local_ip[3]);
                    let request = HttpRequest::new("GET", &host_header, "/");
                    let request_str = request.build_request(&self.config);
                    
                    if let Ok(()) = socket.write_all(request_str.as_bytes()) {
                        socket.flush().unwrap();
                        
                        let mut buffer = [0u8; 512];
                        match socket.read(&mut buffer) {
                            Ok(len) => {
                                rprintln!("HTTP: ✅ Local server responded on port {} ({} bytes)", port, len);
                                
                                if let Ok(response_preview) = core::str::from_utf8(&buffer[..len.min(100)]) {
                                    rprintln!("HTTP: Response preview: {}", response_preview);
                                }
                                
                                socket.disconnect();
                                return Ok(()); // Found working local service
                            }
                            Err(_) => {
                                rprintln!("HTTP: No response from port {}", port);
                            }
                        }
                    }
                    
                    socket.disconnect();
                }
                Err(_) => {
                    rprintln!("HTTP: Port {} not available", port);
                }
            }
        }
        
        rprintln!("HTTP: ⚠️ No local HTTP services found on common ports");
        Ok(())
    }
}

/// Test HTTP client functionality
pub fn test_http_client<DeviceT>(
    stack: &mut blocking_network_stack::Stack<DeviceT>,
) -> Result<(), &'static str>
where
    DeviceT: smoltcp::phy::Device + 'static,
{
    let config = HttpConfig::default();
    let client = SimpleHttpClient::new(config);
    
    rprintln!("HTTP: Starting connectivity tests");
    
    // Test external connectivity
    if let Err(e) = client.test_external_connectivity(stack) {
        rprintln!("HTTP: External test failed: {}", e);
    }
    
    // Small delay between tests
    for _ in 0..500000 { unsafe { core::ptr::read_volatile(&0); } }
    
    // Test local connectivity with broker IP
    if let Err(e) = client.test_local_connectivity(stack, [10, 10, 10, 210]) {
        rprintln!("HTTP: Local test failed: {}", e);
    }
    
    rprintln!("HTTP: Client tests completed");
    Ok(())
}