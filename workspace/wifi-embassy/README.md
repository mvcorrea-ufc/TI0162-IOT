# WiFi Embassy - Conectividade WiFi Assíncrona

## 📡 Descrição

Módulo completo e funcional para conectividade WiFi usando o framework Embassy para ESP32-C3. Implementa conexão WiFi robusta com reconexão automática, aquisição DHCP e stack de rede completo para operações TCP/UDP.

**Status**: ✅ Implementado e testado

## 🚀 Características

- ✅ **Conectividade WiFi Robusta**: Conexão automática com lógica de retry
- ✅ **Suporte DHCP**: Aquisição automática de endereço IP (testado: 10.10.10.214)
- ✅ **Integração Embassy**: Suporte completo async/await com framework Embassy
- ✅ **Reconexão Automática**: Gerencia desconexões de rede graciosamente
- ✅ **Monitoramento de Conexão**: Verificação e relatório de status em tempo real
- ✅ **Acesso Network Stack**: Fornece stack embassy-net para operações TCP/UDP
- ✅ **Arquitetura Comprovada**: Baseado em exemplos funcionais do workspace
- ✅ **Credenciais via Ambiente**: Configuração segura via .cargo/config.toml

## 🏗️ Arquitetura

Este módulo segue os padrões estabelecidos de:
- **bme280-embassy**: Inicialização de hardware e integração Embassy
- **wifi-simple-embassy**: Design de API limpo e tratamento de erros
- **wifi-simple-must-working**: Gerenciamento de conexão assíncrona comprovado

### Estrutura do Projeto

```
wifi-embassy/
├── src/
│   ├── lib.rs              # Interface pública do módulo
│   └── wifi_manager.rs     # Gerenciador WiFi principal
├── examples/
│   ├── wifi_test.rs        # Teste básico de conectividade
│   ├── wifi_test_new.rs    # Teste com informações detalhadas
│   └── wifi_mqtt_test.rs   # Integração WiFi + MQTT completa
├── .cargo/
│   └── config.toml         # Credenciais WiFi via variáveis de ambiente
└── Cargo.toml              # Dependências Embassy

## ⚙️ Configuração

### Credenciais WiFi

Edite `.cargo/config.toml` para configurar suas credenciais:

```toml
[env]
WIFI_SSID = "SuaRedeWiFi"
WIFI_PASSWORD = "SuaSenhaWiFi"
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
```

### Teste de Conectividade Básica

```bash
# Navegar para o módulo
cd wifi-embassy/

# Teste básico de WiFi
cargo run --example wifi_test --release

# Teste com informações detalhadas de rede
cargo run --example wifi_test_new --release

# Teste integrado WiFi + MQTT
cargo run --example wifi_mqtt_test --release
```

### Uso Programático

```rust
use wifi_embassy::{WiFiManager, WiFiConfig};
use embassy_executor::Spawner;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Inicializar ESP32-C3
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Configurar WiFi via variáveis de ambiente
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };
    
    // Criar WiFi manager
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await?;
    
    // WiFi conectado e pronto!
    let stack = wifi_manager.get_stack();
    
    // Usar stack para operações TCP/UDP
}
```

### Monitoramento de Conexão

```rust
// Verificar status da conexão
if let Some(connection_info) = wifi_manager.get_connection_info() {
    rprintln!("📍 IP Address: {}", connection_info.ip_address);
    rprintln!("🌐 Gateway: {:?}", connection_info.gateway);
    rprintln!("🔧 Subnet: /{}", connection_info.subnet_prefix);
}
```

## 📊 Saída Esperada

```
🚀 ESP32-C3 WiFi Embassy Test
📡 Target SSID: FamiliaFeliz-2Ghz
✅ Embassy time driver initialized
🔧 Hardware initialized, starting WiFi connection...
✅ WiFi manager initialized successfully!

🎉 WiFi Connected Successfully!
📡 Network Details:
  📍 IP Address: 10.10.10.214
  🌐 Gateway: Some(10.10.10.1)
  🔧 Subnet: /24
```

## 🔗 Integração com Outros Módulos

### Com BME280 Embassy

```rust
// Inicializar WiFi e BME280 juntos
let wifi_manager = WiFiManager::new(/* params */).await?;
let bme280 = BME280::new(&mut i2c);

// Usar ambos módulos em conjunto
let stack = wifi_manager.get_stack();
let measurements = bme280.read_measurements().await?;

// Enviar dados do sensor via rede
```

### Network Stack para MQTT/HTTP

```rust
let stack = wifi_manager.get_stack();

// O stack pode ser usado com:
// - embassy-net TcpSocket para clientes HTTP
// - Clientes MQTT que aceitam embassy-net stack
// - Aplicações TCP/UDP customizadas
```

## 📋 Requisitos de Hardware

- **ESP32-C3**: Microcontrolador alvo principal
- **Rede WiFi**: Rede 2.4GHz (5GHz não suportada pelo ESP32-C3)
- **Alimentação**: 3.3V estável
- **Antena**: Antena PCB integrada ou externa

## 📦 Dependências

```toml
[dependencies]
# ESP32-C3 Hardware Abstraction Layer
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }

# WiFi Hardware e Network Stack
esp-wifi = { version = "0.15.0", features = ["esp32c3", "wifi", "smoltcp"] }
esp-alloc = { version = "0.8.0" }

# Embassy Async Framework
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-net = { version = "0.7", features = ["tcp", "udp", "dhcpv4", "medium-ethernet"] }
embassy-time = { version = "0.4" }
```

## 🐛 Troubleshooting

### Problemas Comuns

1. **WiFi não conecta**:
   ```bash
   # Verificar credenciais em .cargo/config.toml
   # Verificar se rede é 2.4GHz (não 5GHz)
   # Confirmar SSID exato (case-sensitive)
   ```

2. **DHCP falha**:
   ```bash
   # Verificar router/gateway funcionando
   # Confirmar pool DHCP disponível
   # Testar com dispositivo móvel primeiro
   ```

3. **Embassy time driver não inicializado**:
   ```bash
   # Erro: schedule_wake called before esp_hal_embassy::init()
   # Solução: Chamar esp_hal_embassy::init() antes de WiFiManager::new()
   ```

4. **Build falha**:
   ```bash
   cargo clean
   cargo build --release
   ```

### Debug WiFi

```rust
// Adicionar debug detalhado
rprintln!("WiFi Status: {:?}", wifi_controller.status());
rprintln!("IP Config: {:?}", stack.config_v4());
```

## 🔗 Integração Testada

Este módulo foi testado e integra perfeitamente com:

- **mqtt-embassy**: Publicação MQTT via WiFi (exemplo wifi_mqtt_test.rs)
- **Mosquitto Broker**: Broker MQTT em 10.10.10.210:1883
- **Network Stack**: embassy-net para TCP/UDP

### Exemplo de Integração MQTT

```rust
// Exemplo funcional em examples/wifi_mqtt_test.rs
let stack = wifi_manager.get_stack();
let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);

// Conectar ao broker MQTT
let broker_addr = ("10.10.10.210".parse().unwrap(), 1883);
socket.connect(broker_addr).await?;

// Publicar dados via MQTT
let json_payload = format!(r#"{{"temperature":{:.1},"humidity":{:.1}}}"#, temp, hum);
socket.write_all(&mqtt_publish_packet).await?;
```

## 📄 Licença

MIT OR Apache-2.0

## 👨‍💻 Autor

Marcelo Correa <mvcorrea@gmail.com>