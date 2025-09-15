# Serial Console Embassy - Console Serial Interativo

## 💻 Descrição

Módulo de console serial interativo para ESP32-C3 usando o framework Embassy. Fornece uma interface de linha de comando via UART para configuração e monitoramento do sistema IoT, permitindo configuração dinâmica de credenciais WiFi, MQTT e visualização de status em tempo real.

**Status**: ✅ Implementado e testado

## 🚀 Características

- ✅ **Interface UART Assíncrona**: Console serial via UART0 a 115200 baud
- ✅ **Sistema de Comandos**: Parser robusto com validação de comandos
- ✅ **Configuração Dinâmica**: WiFi e MQTT configuráveis via comandos
- ✅ **Monitoramento em Tempo Real**: Status de sistema e módulos
- ✅ **Integração Embassy**: Async tasks para I/O não bloqueante
- ✅ **Persistência**: Salvar/carregar configurações (preparado para flash)
- ✅ **Modularidade**: Features opcionais para integração seletiva

## 🔌 Hardware e Conexão

### Pinagem UART
```
ESP32-C3        Conversor USB-Serial
--------        -------------------
GPIO1 (TX)  --> RX
GPIO3 (RX)  <-- TX  
GND         --- GND
```

### Configuração do Terminal
- **Baud Rate**: 115200
- **Data Bits**: 8
- **Parity**: None
- **Stop Bits**: 1
- **Flow Control**: None

## 🚀 Uso Rápido

### Instalação e Execução

```bash
# Navegar para o módulo
cd serial-console-embassy/

# Console básico (standalone)
cargo run --example basic_console --release

# Console com integração IoT completa
cargo run --example system_console --features full --release

# Console com features específicas
cargo run --example system_console --features wifi,sensor --release
```

### Conectar via Terminal Serial

```bash
# Linux/macOS
screen /dev/ttyUSB0 115200
# ou
minicom -D /dev/ttyUSB0 -b 115200

# Windows
putty -serial COM3 -serspeed 115200
```

## 📋 Comandos Disponíveis

### Comandos de Sistema
```bash
help, h, ?          # Mostrar ajuda completa
status, stat        # Status atual do sistema
info, i             # Informações detalhadas do hardware
clear, cls          # Limpar tela do terminal
restart, reset      # Reiniciar sistema
```

### Comandos WiFi
```bash
wifi show           # Mostrar configuração atual
wifi ssid <nome>    # Configurar SSID da rede
wifi pass <senha>   # Configurar senha WiFi
```

### Comandos MQTT
```bash
mqtt show           # Mostrar configuração MQTT
mqtt broker <ip>    # Configurar IP do broker
mqtt port <porta>   # Configurar porta (padrão: 1883)
mqtt client <id>    # Configurar client ID
mqtt prefix <pfx>   # Configurar prefixo dos tópicos
```

### Comandos de Configuração
```bash
save                # Salvar configuração na flash
load                # Carregar configuração da flash
```

## 📊 Exemplo de Sessão

```
╔══════════════════════════════════════════════════════════════╗
║              ESP32-C3 IoT System Console                     ║
║                    Embassy Framework                         ║
╚══════════════════════════════════════════════════════════════╝

Type 'help' for available commands

esp32> status
=== System Status ===
WiFi: Connected (10.10.10.214)
MQTT: Connected
Sensor: Active

esp32> wifi show
=== WiFi Configuration ===
SSID: MinhaRedeWiFi
Password: ********
Status: Valid

esp32> mqtt show
=== MQTT Configuration ===
Broker: 10.10.10.210:1883
Client ID: esp32-c3-console
Topic Prefix: esp32
Status: Valid

esp32> wifi ssid NovaRede
WiFi SSID set to: NovaRede

esp32> save
Configuration saved to flash

esp32> info
=== System Information ===
Chip: ESP32-C3
Framework: Embassy
Build: Release
Free Heap: 48KB
```

## 🏗️ Arquitetura do Módulo

### Estrutura de Arquivos

```
serial-console-embassy/
├── src/
│   ├── lib.rs              # Interface pública do módulo
│   ├── console.rs          # Console UART assíncrono
│   ├── commands.rs         # Parser e handler de comandos
│   └── config.rs           # Estruturas de configuração
├── examples/
│   ├── basic_console.rs    # Console básico standalone
│   └── system_console.rs   # Console integrado com IoT
├── .cargo/
│   └── config.toml         # Configuração de build e env vars
├── Cargo.toml              # Dependências e features
└── README.md               # Esta documentação
```

### Features Disponíveis

```toml
[features]
default = []
wifi = ["dep:wifi-embassy"]      # Integração com WiFi
mqtt = ["dep:mqtt-embassy"]      # Integração com MQTT
sensor = ["dep:bme280-embassy"]  # Integração com sensores
usb = ["dep:embassy-usb"]        # Console via USB (futuro)
full = ["wifi", "mqtt", "sensor"] # Todas as features
```

## 🔧 Configuração

### Dependências Principais

```toml
[dependencies]
# ESP32-C3 HAL + Embassy
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }

# Embassy Framework  
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embassy-sync = { version = "0.7" }

# String processing
heapless = "0.8"
embedded-io-async = "0.6"
```

### Ambiente de Desenvolvimento

```toml
# .cargo/config.toml
[env]
WIFI_SSID = "ESP32-Test"
WIFI_PASSWORD = "password123"
MQTT_BROKER_IP = "192.168.1.100"
MQTT_BROKER_PORT = "1883"
```

## 📚 Integração com Outros Módulos

### Com WiFi Embassy
```rust
use wifi_embassy::{WiFiManager, WiFiConfig};
use serial_console_embassy::SerialConsole;

// Atualizar status WiFi no console
console.update_system_status(true, false, true, Some("10.10.10.214")).await;
```

### Com MQTT Embassy
```rust
use mqtt_embassy::MqttClient;

// Configurar MQTT via console e usar no cliente
let config = console.get_config().await;
let mqtt_client = MqttClient::new_from_console_config(&config.mqtt);
```

### Com BME280 Embassy
```rust
use bme280_embassy::BME280;

// Monitorar sensor e reportar status
let sensor_active = bme280.check_id().await.is_ok();
console.update_system_status(wifi_ok, mqtt_ok, sensor_active, ip).await;
```

## 🐛 Troubleshooting

### Problemas Comuns

1. **Console não responde**:
   ```bash
   # Verificar conexão serial
   # Confirmar baud rate 115200
   # Testar com diferentes terminais
   ```

2. **Caracteres não aparecem**:
   ```bash
   # Verificar TX/RX não invertidos
   # Confirmar GND comum
   # Testar cabo USB-serial
   ```

3. **Build falha**:
   ```bash
   cargo clean
   cargo build --example basic_console --release
   ```

4. **Features não disponíveis**:
   ```bash
   # Usar features corretas
   cargo run --example system_console --features full --release
   ```

### Debug do Console

```rust
// Logs RTT para debug do console
rprintln!("[CONSOLE] Command received: {}", command);
rprintln!("[CONSOLE] Status updated: WiFi={}, MQTT={}", wifi, mqtt);
```

## 🔮 Extensões Futuras

### Recursos Planejados
- **Flash Storage**: Persistência real de configurações
- **Command History**: Histórico de comandos com setas
- **Auto-completion**: Completar comandos automaticamente
- **USB Console**: Console via USB CDC em adição ao UART
- **Web Console**: Interface web para comando remoto
- **Scripting**: Execução de scripts de comandos

### Comandos Adicionais
- **log level**: Configurar nível de logging
- **network scan**: Escanear redes WiFi disponíveis
- **sensor calibrate**: Calibração manual de sensores
- **system update**: Atualização OTA via console

## 📄 Licença

MIT OR Apache-2.0

## 👨‍💻 Autor

Marcelo Correa <mvcorrea@gmail.com>

**Projeto TI0162 - Internet das Coisas**  
**Console Serial Interativo para Sistema IoT ESP32-C3**