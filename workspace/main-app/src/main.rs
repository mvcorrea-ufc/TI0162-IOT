#![no_std]
#![no_main]

extern crate alloc;

use esp_println::println;
use esp_backtrace as _;

use esp_hal::{
    delay::Delay,
    prelude::*,
    rng::Rng,
    timer::systimer::{SystemTimer, Target},
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_wifi::{
    initialize,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitFor,
};
use embassy_net::{Stack, StackResources, Config};
use static_cell::StaticCell;

use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("Main App: Starting WiFi connection task");
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.try_into().unwrap(),
                password: WIFI_PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Main App: Starting WiFi controller");
            controller.start().await.unwrap();
            println!("Main App: WiFi started!");
        }
        
        println!("Main App: About to connect to '{}'", WIFI_SSID);
        match controller.connect().await {
            Ok(_) => println!("Main App: Connected to WiFi!"),
            Err(e) => {
                println!("Main App: Failed to connect: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

#[embassy_executor::task]  
async fn mqtt_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    println!("Main App: Starting MQTT task");
    
    // Wait for network
    stack.wait_config_up().await;
    
    if let Some(config) = stack.config_v4() {
        println!("Main App: Network ready, IP: {}", config.address.address());
    }
    
    let mqtt_config = MqttConfig::default();
    let client = MqttClient::new(mqtt_config);
    
    println!("Main App: Connecting to MQTT broker at {}:{}", 
             mqtt_config.broker_ip, mqtt_config.broker_port);
    
    // Create socket buffers
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    // MQTT publishing loop
    let mut counter = 0;
    loop {
        counter += 1;
        
        // Create mock sensor data
        let temperature = 23.0 + (counter as f32 * 0.1);
        let humidity = 65.0 - (counter as f32 * 0.2);
        let pressure = 1013.25 + (counter as f32 * 0.05);
        
        let sensor_data = SensorData::new(temperature, humidity, pressure);
        
        println!("Main App: Mock sensor reading #{} - T: {:.1}°C, H: {:.1}%, P: {:.1} hPa", 
                counter, temperature, humidity, pressure);
        
        // Connect and publish
        match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
            Ok(mut socket) => {
                println!("Main App: MQTT connected");
                
                // Publish sensor data
                match client.publish_sensor_data(&mut socket, &sensor_data).await {
                    Ok(_) => println!("Main App: Sensor data published successfully!"),
                    Err(e) => println!("Main App: Failed to publish: {}", e),
                }
                
                // Publish device status every 5 readings
                if counter % 5 == 0 {
                    let device_status = DeviceStatus::new("online", counter * 30, 50000, -45);
                    match client.publish_device_status(&mut socket, &device_status).await {
                        Ok(_) => println!("Main App: Device status published"),
                        Err(e) => println!("Main App: Device status failed: {}", e),
                    }
                }
                
                // Heartbeat every 10 readings
                if counter % 10 == 0 {
                    match client.publish_heartbeat(&mut socket).await {
                        Ok(_) => println!("Main App: Heartbeat published"),
                        Err(e) => println!("Main App: Heartbeat failed: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("Main App: MQTT connection failed: {}", e);
            }
        }
        
        // Wait 30 seconds between readings
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    println!("=== ESP32-C3 IoT Main Application ===");
    println!("Modules: WiFi + MQTT");
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    let systimer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        systimer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &esp_hal::clock::ClockControl::max(peripherals.SYSTEM.split().clock_control).freeze(),
    ).unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    // Initialize Embassy
    let timer_group0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(
        &esp_hal::clock::ClockControl::max(peripherals.SYSTEM.split().clock_control).freeze(),
        timer_group0.timer0,
    );

    // Network stack
    static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let stack_resources = STACK_RESOURCES.init(StackResources::<3>::new());
    static STACK: StaticCell<Stack<WifiDevice<'static, WifiStaDevice>>> = StaticCell::new();
    let stack = STACK.init(Stack::new(
        wifi_interface,
        Config::dhcpv4(Default::default()),
        stack_resources,
        embassy_net::random_seed(),
    ));

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(stack)).ok();
    spawner.spawn(mqtt_task(stack)).ok();

    println!("Main App: All tasks spawned");

    loop {
        Timer::after(Duration::from_secs(60)).await;
        println!("Main App: System heartbeat");
    }
}