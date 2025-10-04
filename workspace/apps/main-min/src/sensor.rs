//! Sensor Management - Single file module

use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::I2c;

#[cfg(feature = "sensor")]
use bme280_embassy::{BME280, I2cDevice};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct SensorData {
    pub temperature: f32,
    pub pressure: f32,
    pub humidity: f32,
    pub count: u32,
}

pub struct SensorManager;

impl SensorManager {
    #[cfg(feature = "sensor")]
    pub async fn run(
        i2c: &'static mut I2c<'static, esp_hal::Blocking>,
        interval_secs: u32,
        sensor_signal: &'static embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, SensorData>,
    ) -> ! {
        rtt_target::rprintln!("Sensor: Starting BME280...");

        // Initialize BME280 exactly like main-app
        let i2c_device = I2cDevice::new(i2c, 0x76);
        let mut bme280 = BME280::new(i2c_device);

        // Retry initialization 
        loop {
            match bme280.init().await {
                Ok(_) => {
                    rtt_target::rprintln!("BME280 ready");
                    break;
                }
                Err(_) => {
                    rtt_target::rprintln!("BME280 init failed, retry in 5s");
                    Timer::after(Duration::from_secs(5)).await;
                }
            }
        }

        let mut count = 0u32;
        loop {
            match bme280.read_measurements().await {
                Ok(measurements) => {
                    count += 1;
                    rtt_target::rprintln!(
                        "[SENSOR] #{}: T={:.2}°C P={:.1}hPa H={:.1}%",
                        count,
                        measurements.temperature,
                        measurements.pressure,
                        measurements.humidity
                    );
                    
                    // Send sensor data to MQTT task via signal
                    let sensor_data = SensorData {
                        temperature: measurements.temperature,
                        pressure: measurements.pressure,
                        humidity: measurements.humidity,
                        count,
                    };
                    sensor_signal.signal(sensor_data);
                    rtt_target::rprintln!("[SENSOR] Data sent to MQTT task");
                }
                Err(_) => {
                    rtt_target::rprintln!("[SENSOR] Read error");
                }
            }

            Timer::after(Duration::from_secs(interval_secs as u64)).await;
        }
    }

    #[cfg(not(feature = "sensor"))]
    pub async fn run(
        _i2c: &'static mut I2c<'static, esp_hal::Blocking>,
        _interval_secs: u32,
        _sensor_signal: &'static embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, SensorData>,
    ) -> ! {
        rtt_target::rprintln!("Sensor: Disabled");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
}