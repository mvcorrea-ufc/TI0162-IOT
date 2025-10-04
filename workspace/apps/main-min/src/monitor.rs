//! System monitoring - Minimal status reporting

use embassy_time::{Duration, Timer};

pub struct SystemMonitor;

impl SystemMonitor {
    pub async fn run() -> ! {
        let mut count = 0u32;
        
        loop {
            Timer::after(Duration::from_secs(300)).await; // 5 minutes
            count += 1;
            rtt_target::rprintln!("Status #{}: System operational", count);
        }
    }
}