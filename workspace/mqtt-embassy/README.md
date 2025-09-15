# MQTT Embassy - Cliente MQTT Assíncrono

## 📨 Descrição

Módulo completo e funcional para cliente MQTT usando o framework Embassy para ESP32-C3. Implementa cliente MQTT assíncrono via Embassy TCP sockets com suporte a publicação JSON de dados de sensores, status e heartbeat.

**Status**: ✅ Implementado e testado

## 🚀 Características

- ✅ **Cliente MQTT Assíncrono**: Via Embassy TCP sockets
- ✅ **Protocolo MQTT 3.1.1**: Implementação completa do protocolo
- ✅ **Broker Configurável**: Suporte a broker via variáveis de ambiente (testado: 10.10.10.210:1883)
- ✅ **Publicação JSON**: Dados estruturados de sensores, status e heartbeat
- ✅ **Configuração via Ambiente**: Credenciais seguras via .cargo/config.toml
- ✅ **Integração WiFi**: Funciona perfeitamente com wifi-embassy
- ✅ **Reconexão Robusta**: Criação de nova conexão para cada ciclo de publicação
- ✅ **Pipeline IoT Completo**: ESP32-C3 → WiFi → MQTT → Subscribers

## 🏗️ Arquitetura

### Estrutura do Projeto

```
mqtt-embassy/
├── src/
│   ├── lib.rs              # Interface pública do módulo
│   ├── mqtt_client.rs      # Cliente MQTT principal
│   └── message.rs          # Estruturas de mensagem JSON
├── examples/
│   ├── mqtt_test.rs        # Teste básico MQTT
│   └── mqtt_test_working.rs # Teste integrado com WiFi
├── .cargo/
│   └── config.toml         # Configuração do broker via env vars
└── Cargo.toml              # Dependências Embassy
```

### Fluxo de Dados

```
ESP32-C3 → WiFi → MQTT Broker → Mosquitto Subscribers
         ↑                    ↑
   wifi-embassy        mqtt-embassy
```

## ⚙️ Configuração

### Broker MQTT

Edite `.cargo/config.toml` para configurar o broker:

```toml
[env]
WIFI_SSID = "SuaRedeWiFi"
WIFI_PASSWORD = "SuaSenhaWiFi"
MQTT_BROKER_IP = "10.10.10.210"
MQTT_BROKER_PORT = "1883"
MQTT_CLIENT_ID = "esp32-c3-iot"
MQTT_TOPIC_PREFIX = "esp32"
```

### Mosquitto Broker

```bash
# Instalar Mosquitto
sudo apt install mosquitto mosquitto-clients

# Iniciar broker
sudo systemctl start mosquitto

# Configurar para aceitar conexões remotas
sudo nano /etc/mosquitto/mosquitto.conf
# Adicionar:
# listener 1883 0.0.0.0
# allow_anonymous true

# Reiniciar
sudo systemctl restart mosquitto
```

## 🚀 Uso Rápido

### Pré-requisitos

```bash
# Instalar target Rust para ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Instalar probe-rs
cargo install probe-rs --features cli

# Verificar dispositivo conectado
probe-rs list

# Verificar broker MQTT disponível
ping 10.10.10.210
```

### Teste MQTT

```bash
# Navegar para o módulo
cd mqtt-embassy/

# Terminal 1: Monitor MQTT (antes de executar o ESP32)
mosquitto_sub -h 10.10.10.210 -p 1883 -t "esp32/#" -v

# Terminal 2: Executar ESP32
cargo run --example mqtt_test_working --features examples --release
```

### Uso Programático

```rust
use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};
use wifi_embassy::WiFiManager;

#[embassy_executor::task]
async fn mqtt_task(wifi_manager: &'static WiFiManager) {
    // Configurar MQTT
    let mqtt_config = MqttConfig::default();
    let client = MqttClient::new(mqtt_config);
    
    // Obter network stack do WiFi
    let stack = wifi_manager.get_stack();
    
    // Criar dados do sensor
    let sensor_data = SensorData::new(23.5, 68.2, 1013.8);
    
    // Buffers para conexão TCP
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    // Conectar e publicar
    match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
        Ok(mut socket) => {
            // Publicar dados do sensor
            client.publish_sensor_data(&mut socket, &sensor_data).await?;
            
            // Publicar heartbeat
            client.publish_heartbeat(&mut socket).await?;
        }
        Err(e) => rprintln!("Erro MQTT: {}", e),
    }
}
```

## 📊 Mensagens Publicadas

### Dados do Sensor (esp32/sensor/bme280)

```json
{
  "temperature": 23.5,
  "humidity": 68.2,
  "pressure": 1013.8,
  "reading": 1
}
```

### Status do Dispositivo (esp32/status)

```json
{
  "status": "online",
  "uptime": 300,
  "free_heap": 45000,
  "wifi_rssi": -42
}
```

### Heartbeat (esp32/heartbeat)

```
ping
```

## 📊 Saída Esperada

### Console ESP32

```
🚀 ESP32-C3 MQTT Embassy Test
📡 WiFi + MQTT Integration Test
Target SSID: FamiliaFeliz-2Ghz
MQTT Broker: 10.10.10.210:1883
✅ Embassy time driver initialized successfully
✅ WiFi manager initialized successfully!

🎉 WiFi Connected Successfully!
📡 Network Details:
  📍 IP Address: 10.10.10.214
  🌐 Gateway: Some(10.10.10.1)
  🔧 Subnet: /24

MQTT Task: Reading #1 - T: 22.1°C, H: 68.0%, P: 1013.3 hPa
MQTT Task: ✅ Connected to MQTT broker successfully!
MQTT Task: ✅ Sensor data published to topic 'esp32/sensor/bme280'
```

### Mosquitto Monitor

```bash
$ mosquitto_sub -h 10.10.10.210 -p 1883 -t "esp32/#" -v

esp32/sensor/bme280 {"temperature":22.1,"humidity":68.0,"pressure":1013.3,"reading":1}
esp32/sensor/bme280 {"temperature":22.2,"humidity":67.8,"pressure":1013.4,"reading":2}
esp32/heartbeat ping
esp32/status {"status":"online","uptime":300,"free_heap":48000,"wifi_rssi":-38}
```

## 🔗 Integração Testada

### Com WiFi Embassy

Exemplo funcional disponível em `wifi-embassy/examples/wifi_mqtt_test.rs`:

```rust
// Sistema completo WiFi + MQTT
let wifi_manager = WiFiManager::new(/* params */).await?;
let stack = wifi_manager.get_stack();

// Publicação MQTT direta via TCP sockets
let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
let broker_addr = ("10.10.10.210".parse().unwrap(), 1883);
socket.connect(broker_addr).await?;

// Enviar MQTT CONNECT e PUBLISH
socket.write_all(&connect_packet).await?;
socket.write_all(&publish_packet).await?;
```

### Com BME280 Embassy

```rust
// Integração com sensor real
let measurements = bme280.read_measurements().await?;
let sensor_data = SensorData::new(
    measurements.temperature,
    measurements.humidity,
    measurements.pressure
);
client.publish_sensor_data(&mut socket, &sensor_data).await?;
```

## 📦 Dependências

```toml
[dependencies]
# ESP32-C3 Hardware Abstraction Layer
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }

# WiFi Embassy (integração)
wifi-embassy = { path = "../wifi-embassy" }

# Embassy Async Framework
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embedded-io-async = "0.6"

# JSON e utilidades
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"
heapless = "0.8"
```

## 🐛 Troubleshooting

### Problemas Comuns

1. **Broker MQTT não acessível**:
   ```bash
   # Verificar conectividade
   ping 10.10.10.210
   telnet 10.10.10.210 1883
   
   # Verificar configuração Mosquitto
   sudo systemctl status mosquitto
   sudo journalctl -u mosquitto
   ```

2. **Mensagens não aparecem no subscriber**:
   ```bash
   # Verificar formato do pacote MQTT
   # Adicionar debug hex no código
   rprintln!("MQTT Packet: {:02X?}", &publish_packet);
   
   # Verificar tópicos
   mosquitto_sub -h 10.10.10.210 -t "#" -v
   ```

3. **WiFi conectado mas MQTT falha**:
   ```bash
   # Verificar stack de rede
   let stack = wifi_manager.get_stack();
   rprintln!("Stack status: {:?}", stack.config_v4());
   ```

4. **Build falha**:
   ```bash
   cargo clean
   cargo build --example mqtt_test_working --features examples --release
   ```

### Debug MQTT

```rust
// Debug detalhado do protocolo MQTT
rprintln!("MQTT CONNECT packet: {:02X?}", &connect_packet);
rprintln!("MQTT PUBLISH packet: {:02X?}", &publish_packet[..20]);
rprintln!("Socket state: {:?}", socket.state());
```

## 📋 Especificações MQTT

- **Protocolo**: MQTT 3.1.1
- **QoS**: 0 (Fire and forget)
- **Retain**: false
- **Keep Alive**: 60 segundos
- **Clean Session**: true
- **Client ID**: Configurável via env var

### Formato dos Pacotes

```
CONNECT:  [0x10, length, protocol_name, version, flags, keep_alive, client_id]
PUBLISH:  [0x30, length, topic_length, topic, payload]
```

## 🔄 Ciclo de Publicação

1. **Sensor Data**: A cada 30 segundos
2. **Heartbeat**: A cada 5 ciclos (2.5 minutos)
3. **Device Status**: A cada 10 ciclos (5 minutos)

## 📄 Licença

MIT OR Apache-2.0

## 👨‍💻 Autor

Marcelo Correa <mvcorrea@gmail.com>