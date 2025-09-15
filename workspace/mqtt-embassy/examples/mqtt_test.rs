#![no_std]
#![no_main]

extern crate alloc;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use esp_hal::{
    gpio::{Io, Level, Output, OutputConfig},
    rng::Rng,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
        WifiState,
    },
    EspWifiController, WifiStaDevice,
};

use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources, Config};
use embassy_time::{Duration, Timer};

use static_cell::StaticCell;

// MQTT Embassy imports
use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};

const WIFI_SSID: &str = env!("WIFI_SSID", "Set WIFI_SSID in .cargo/config.toml");
const WIFI_PASS: &str = env!("WIFI_PASSWORD", "Set WIFI_PASSWORD in .cargo/config.toml");

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    rprintln!("WiFi Embassy: Starting connection task");
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await;
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.try_into().unwrap(),
                password: WIFI_PASS.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            rprintln!("WiFi Embassy: Starting WiFi controller");
            controller.start().unwrap();
            rprintln!("WiFi Embassy: WiFi started!");
        }
        rprintln!("WiFi Embassy: About to connect to WiFi");
        match controller.connect() {
            Ok(_) => rprintln!("WiFi Embassy: Connected to WiFi!"),
            Err(e) => {
                rprintln!("WiFi Embassy: Failed to connect to WiFi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<'static>) {
    stack.run().await
}

#[embassy_executor::task]
async fn mqtt_test_task(stack: &'static Stack<'static>) {
    rprintln!("MQTT Embassy Test: Starting MQTT test task");
    
    // Wait for network to be ready
    stack.wait_config_up().await;
    
    if let Some(config) = stack.config_v4() {
        rprintln!("MQTT Embassy Test: Network ready, IP: {}", config.address.address());
    }
    
    // Configure MQTT client
    let mqtt_config = MqttConfig::default();
    let client = MqttClient::new(mqtt_config);
    
    rprintln!("MQTT Embassy Test: Connecting to MQTT broker at {}:{}...", 
             mqtt_config.broker_ip, mqtt_config.broker_port);
    
    // Create buffers for socket
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    // Connect to MQTT broker
    match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
        Ok(mut socket) => {
            rprintln!("MQTT Embassy Test: Successfully connected to MQTT broker!");
            
            // Test sensor data publishing
            let test_sensor_data = SensorData::new(24.5, 65.8, 1015.2);
            rprintln!("MQTT Embassy Test: Publishing sensor data: T={}°C, H={}%, P={}hPa", 
                     test_sensor_data.temperature, 
                     test_sensor_data.humidity, 
                     test_sensor_data.pressure);
            
            match client.publish_sensor_data(&mut socket, &test_sensor_data).await {
                Ok(_) => rprintln!("MQTT Embassy Test: Sensor data published successfully!"),
                Err(e) => rprintln!("MQTT Embassy Test: Failed to publish sensor data: {}", e),
            }
            
            // Test device status publishing
            let device_status = DeviceStatus::new("online", 120, 45000, -42);
            rprintln!("MQTT Embassy Test: Publishing device status: {}, uptime: {}s", 
                     device_status.status, device_status.uptime);
            
            match client.publish_device_status(&mut socket, &device_status).await {
                Ok(_) => rprintln!("MQTT Embassy Test: Device status published successfully!"),
                Err(e) => rprintln!("MQTT Embassy Test: Failed to publish device status: {}", e),
            }
            
            // Test heartbeat
            match client.publish_heartbeat(&mut socket).await {
                Ok(_) => rprintln!("MQTT Embassy Test: Heartbeat published successfully!"),
                Err(e) => rprintln!("MQTT Embassy Test: Failed to publish heartbeat: {}", e),
            }
            
            // Keep connection alive and publish periodically
            let mut counter = 0;
            loop {
                Timer::after(Duration::from_secs(15)).await;
                counter += 1;
                
                // Update test data with counter
                let updated_sensor_data = SensorData::new(
                    24.5 + (counter as f32 * 0.1), 
                    65.8 - (counter as f32 * 0.2), 
                    1015.2 + (counter as f32 * 0.05)
                );
                
                match client.publish_sensor_data(&mut socket, &updated_sensor_data).await {
                    Ok(_) => rprintln!("MQTT Embassy Test: Periodic sensor data #{} published", counter),
                    Err(e) => {
                        rprintln!("MQTT Embassy Test: Failed to publish periodic data: {}", e);
                        break; // Exit loop on error
                    }
                }
                
                // Publish heartbeat every 3 cycles (45 seconds)
                if counter % 3 == 0 {
                    match client.publish_heartbeat(&mut socket).await {
                        Ok(_) => rprintln!("MQTT Embassy Test: Periodic heartbeat published"),
                        Err(e) => rprintln!("MQTT Embassy Test: Heartbeat error: {}", e),
                    }
                }
            }
        }
        Err(e) => {
            rprintln!("MQTT Embassy Test: Failed to connect to MQTT broker: {}", e);
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize heap allocator (required for WiFi)
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    rtt_init_print!();
    rprintln!("MQTT Embassy Test: Starting ESP32-C3 MQTT test");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Setup LED for status indication
    let io = Io::new(peripherals.IO_MUX);
    let mut _led = Output::new(io.pins.gpio8, Level::Low, OutputConfig::default());

    // Initialize WiFi
    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = esp_wifi::init(
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    // Initialize Embassy
    esp_hal_embassy::init(TimerGroup::new(peripherals.TIMG1).timer0);

    // Network stack setup
    static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let stack_resources = STACK_RESOURCES.init(StackResources::<3>::new());
    
    static STACK: StaticCell<Stack<'static>> = StaticCell::new();
    let stack = STACK.init(Stack::new(
        wifi_interface,
        Config::dhcpv4(Default::default()),
        stack_resources,
        embassy_net::random_seed(),
    ));

    // Spawn tasks
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(stack)).ok();
    spawner.spawn(mqtt_test_task(stack)).ok();

    rprintln!("MQTT Embassy Test: All tasks spawned, entering main loop");

    // Main loop with status LED blinking
    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}