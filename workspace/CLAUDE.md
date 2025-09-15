# TI0162 - Internet das Coisas - Projeto

## Visão Geral do Projeto

Este é um projeto de Internet das Coisas (IoT) desenvolvido em Rust para ESP32-C3 utilizando o framework Embassy. O projeto implementa um sistema modular de sensoriamento e conectividade, com foco na coleta de dados ambientais e transmissão via WiFi e MQTT.

**Base de Implementação**: O projeto utiliza o exemplo `blinky` como base de implementação, aproveitando sua configuração já estabelecida para ESP32-C3 com esp-hal.

## Tecnologias Utilizadas

- **Linguagem**: Rust
- **Microcontrolador**: ESP32-C3
- **HAL**: esp-hal v0.23.1 (Hardware Abstraction Layer)
- **Framework Async**: Embassy (async framework for embedded systems)
- **Base Template**: Projeto `blinky` do rust-esp32-tmpl
- **Debugging**: RTT (Real-Time Transfer) via rtt-target
- **Sensor**: BME280 (temperatura, umidade e pressão)
- **Conectividade**: WiFi + MQTT
- **Broker MQTT**: Mosquitto

## Decisões Arquiteturais

### ⚡ esp-hal + Embassy (Chosen)
- **Vantagens**: Leve, performático, controle total do hardware
- **Desvantagens**: Mais código manual, menos abstrações
- **Uso**: Ideal para projetos IoT com recursos limitados

### ❌ esp-idf/FreeRTOS (Evitado)
- **Desvantagens**: Framework pesado, mais overhead de memória
- **Razão**: Desnecessário para aplicações simples de IoT
- **Impacto**: Reduz recursos disponíveis para lógica de aplicação

## Arquitetura Modular

O projeto foi estruturado de forma modular baseado no template `blinky`, expandindo suas funcionalidades:

```
workspace/
├── blinky/                 # 🏗️ BASE - Template original com esp-hal e RTT
├── examples/               # 📚 REFERÊNCIAS - Exemplos existentes BME280
│   └── simple-bme280-02/   #     Driver BME280 com embedded-hal 1.0
├── bme280-embassy/         # 🌡️ IMPLEMENTADO - BME280 + Embassy async
├── wifi-embassy/           # 📡 IMPLEMENTADO - WiFi connectivity usando Embassy
├── mqtt-embassy/           # 📨 IMPLEMENTADO - MQTT client usando Embassy
├── serial-console-embassy/ # 💻 IMPLEMENTADO - Console serial interativo
├── web-server/             # 🌐 Servidor web para display dos dados
└── main-app/               # 🎯 Aplicação principal integrando todos os módulos
```

### Estrutura Base (blinky)
- **Cargo.toml**: Configuração com esp-hal 0.23.1, rtt-target para debugging
- **main.rs**: Estrutura base com inicialização RTT e loop principal
- **build.rs**: Configuração de linking necessária para ESP32-C3
- **Funcionalidade**: LED blinking com output RTT para validação

## Funcionalidades Implementadas

### 1. ✅ Módulo BME280 Embassy (`bme280-embassy/`)
- **Status**: Completo e funcional
- Leitura assíncrona de temperatura (°C), umidade (%) e pressão (hPa) 
- Interface I2C assíncrona via Embassy
- Calibração automática do sensor BME280
- Compensação de valores com algoritmos corrigidos
- Output RTT para debugging

### 2. ✅ Módulo WiFi Embassy (`wifi-embassy/`)
- **Status**: Completo e funcional
- Conexão WiFi automática com credenciais via variáveis de ambiente
- Aquisição de IP via DHCP (testado: 10.10.10.214)
- Reconexão automática em caso de desconexão
- Network stack Embassy com suporte TCP/UDP
- Interface de gerenciamento WiFi simplificada

### 3. ✅ Módulo MQTT Embassy (`mqtt-embassy/`)
- **Status**: Completo e funcional  
- Cliente MQTT assíncrono via Embassy TCP sockets
- Suporte a broker configurável (testado: 10.10.10.210:1883)
- Publicação JSON de dados de sensores, status e heartbeat
- Configuração via variáveis de ambiente
- Protocolo MQTT 3.1.1 completo

### 4. ✅ Módulo Serial Console Embassy (`serial-console-embassy/`)
- **Status**: Completo e funcional
- Interface serial interativa via UART usando Embassy async
- Sistema de comandos para configuração e monitoramento
- Configuração dinâmica de credenciais WiFi e MQTT
- Display de informações do sistema em tempo real
- Parser de comandos robusto com validação

### 5. ✅ Integração WiFi + MQTT
- **Status**: Completo e operacional
- Sistema completo ESP32-C3 → WiFi → MQTT → Subscribers
- Publicação periódica de dados (30s sensor, 2.5min heartbeat, 5min status)
- JSON estruturado conforme especificação do projeto
- Pipeline IoT totalmente funcional

## Estrutura de Dados

### Payload MQTT (JSON)
```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "sensor": "BME280",
  "data": {
    "temperature": 23.5,
    "humidity": 65.2,
    "pressure": 1013.25
  }
}
```

## 📖 Guia de Uso dos Módulos e Exemplos

### Pré-requisitos

1. **Rust toolchain**: `rustup target add riscv32imc-unknown-none-elf`
2. **probe-rs**: `cargo install probe-rs --features cli`
3. **ESP32-C3**: Conectado via USB com drivers instalados
4. **WiFi**: Rede disponível para testes de conectividade
5. **MQTT Broker**: Mosquitto ou similar para testes MQTT

### 🌡️ Módulo BME280 Embassy

**Localização**: `bme280-embassy/`

```bash
# Navegar para o módulo
cd bme280-embassy/

# Executar leitura básica do BME280
cargo run --example basic_reading --release

# Executar aplicação principal (leitura contínua)
cargo run --release
```

**Configuração Hardware**:
- BME280: SDA=GPIO8, SCL=GPIO9
- LED de status: GPIO3
- Frequência I2C: 100kHz

**Saída Esperada**:
```
BME280 Embassy: Sensor initialized successfully
BME280 Embassy: T: 23.2°C, H: 68.5%, P: 1013.8 hPa
BME280 Embassy: T: 23.1°C, H: 68.3%, P: 1013.9 hPa
```

### 📡 Módulo WiFi Embassy

**Localização**: `wifi-embassy/`

**Configuração**: Editar `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "SuaRedeWiFi"
WIFI_PASSWORD = "SuaSenhaWiFi"
```

```bash
# Navegar para o módulo
cd wifi-embassy/

# Teste básico de conectividade WiFi
cargo run --example wifi_test --release

# Teste completo com informações de rede
cargo run --example wifi_test_new --release

# Integração WiFi + MQTT (requer broker MQTT)
cargo run --example wifi_mqtt_test --release
```

**Saída Esperada**:
```
WiFi Embassy: Connected to WiFi!
📍 IP Address: 10.10.10.214
🌐 Gateway: Some(10.10.10.1)
🔧 Subnet: /24
```

### 💻 Módulo Serial Console Embassy

**Localização**: `serial-console-embassy/`

```bash
# Navegar para o módulo
cd serial-console-embassy/

# Console básico (sem integração)
cargo run --example basic_console --release

# Console completo com IoT (requer módulos)
cargo run --example system_console --features full --release
```

**Interface Serial**: UART0 a 115200 baud
- **TX**: GPIO1 (conectar ao RX do conversor USB-serial)
- **RX**: GPIO3 (conectar ao TX do conversor USB-serial) 
- **GND**: Comum entre ESP32-C3 e conversor

**Comandos Disponíveis**:

```bash
# Comandos de sistema
help, h, ?          # Mostrar ajuda
status, stat        # Status do sistema
info, i             # Informações detalhadas
clear, cls          # Limpar tela
restart, reset      # Reiniciar sistema

# Comandos WiFi
wifi show           # Mostrar configuração WiFi
wifi ssid <nome>    # Configurar SSID
wifi pass <senha>   # Configurar senha

# Comandos MQTT  
mqtt show           # Mostrar configuração MQTT
mqtt broker <ip>    # Configurar IP do broker
mqtt port <porta>   # Configurar porta
mqtt client <id>    # Configurar client ID
mqtt prefix <pfx>   # Configurar prefixo dos tópicos

# Comandos de configuração
save                # Salvar config na flash
load                # Carregar config da flash
```

**Exemplo de Sessão**:
```
esp32> help
=== ESP32-C3 IoT System Console ===
Available commands:
[lista de comandos...]

esp32> status
=== System Status ===
WiFi: Connected (10.10.10.214)
MQTT: Connected
Sensor: Active

esp32> wifi ssid MinhaRede
WiFi SSID set to: MinhaRede

esp32> mqtt broker 192.168.1.100
MQTT broker set to: 192.168.1.100

esp32> save
Configuration saved to flash
```

### 📨 Módulo MQTT Embassy

**Localização**: `mqtt-embassy/`

**Configuração**: Editar `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "SuaRedeWiFi"
WIFI_PASSWORD = "SuaSenhaWiFi"
MQTT_BROKER_IP = "192.168.1.100"  # IP do seu broker
MQTT_BROKER_PORT = "1883"
MQTT_CLIENT_ID = "esp32-c3-test"
MQTT_TOPIC_PREFIX = "esp32"
```

```bash
# Navegar para o módulo
cd mqtt-embassy/

# Teste MQTT básico (requer wifi-embassy)
cargo run --example mqtt_test_working --features examples --release
```

**Monitor MQTT** (terminal separado):
```bash
# Monitorar todas as mensagens ESP32
mosquitto_sub -h [SEU_BROKER_IP] -p 1883 -t "esp32/#" -v

# Monitorar tópico específico
mosquitto_sub -h [SEU_BROKER_IP] -p 1883 -t "esp32/sensor/bme280" -v
```

**Mensagens MQTT Publicadas**:
```json
// esp32/sensor/bme280
{"temperature":23.5,"humidity":68.2,"pressure":1013.8,"reading":1}

// esp32/status  
{"status":"online","uptime":300,"free_heap":45000,"wifi_rssi":-42}

// esp32/heartbeat
ping
```

### 🚀 Sistema Integrado (WiFi + MQTT)

**Exemplo Recomendado**: `wifi-embassy/examples/wifi_mqtt_test.rs`

```bash
cd wifi-embassy/
cargo run --example wifi_mqtt_test --release
```

**Funcionalidades**:
- ✅ Conexão WiFi automática
- ✅ Publicação MQTT a cada 30 segundos  
- ✅ Heartbeat a cada 5 ciclos (2.5 minutos)
- ✅ Status do dispositivo a cada 10 ciclos (5 minutos)
- ✅ Reconexão automática WiFi e MQTT
- ✅ JSON estruturado conforme especificação

### 🔧 Comandos de Desenvolvimento

```bash
# Build apenas (sem flash)
cargo build --release

# Build e flash com monitor RTT
cargo run --release

# Build exemplo específico
cargo build --example [NOME_EXEMPLO] --release

# Flash exemplo específico
cargo run --example [NOME_EXEMPLO] --release

# Linting
cargo clippy

# Formatação
cargo fmt

# Limpeza
cargo clean
```

### 🐛 Debugging e Troubleshooting

**RTT Debugging**:
- Todas as aplicações usam `rtt-target` para output em tempo real
- Use `rprintln!()` em lugar de `println!()`
- Monitor via probe-rs automaticamente

**Problemas Comuns**:

1. **WiFi não conecta**: Verificar credenciais em `.cargo/config.toml`
2. **MQTT não publica**: Verificar IP do broker e firewall
3. **BME280 não responde**: Verificar pinagem I2C (SDA=GPIO8, SCL=GPIO9)
4. **Build falha**: Executar `cargo clean` e tentar novamente

**Verificação de Hardware**:
```bash
# Verificar se ESP32-C3 está conectado
probe-rs list

# Verificar target Rust
rustup target list | grep riscv32imc
```

### Estrutura de Debugging (RTT)
O projeto utiliza RTT (Real-Time Transfer) para debugging em tempo real:
- **rtt-target**: Output de logs via RTT
- **panic-rtt-target**: Panic handler via RTT  
- **rprintln!()**: Macro para print via RTT (substitui println!)

### Configurações Base (blinky) → Embassy Migration

**Dependências Atuais (blinky)**:
```toml
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
esp-rom-sys = { version = "0.1", features = ["esp32c3"] }
defmt = "0.3"
rtt-target = "0.5"
panic-rtt-target = "0.1"
```

**Dependências Embassy (2025)**:
```toml
# Base Embassy
embassy-executor = { version = "0.7", features = ["task-arena-size-20480"] }
embassy-time = "0.4.0"

# ESP32-C3 HAL + Embassy Integration  
esp-hal = { version = "0.23.1", features = ["esp32c3", "log"] }
esp-hal-embassy = { version = "0.6", features = ["esp32c3"] }

# WiFi Support
esp-wifi = { git = "https://github.com/esp-rs/esp-hal", features = ["esp32c3", "wifi", "embassy-net"] }

# Utilities
esp-backtrace = { version = "0.15.0", features = ["esp32c3", "exception-handler", "panic-handler", "println"] }
esp-println = { version = "0.13.0", features = ["esp32c3", "log"] }

# I2C Async Support
embedded-hal-async = "1.0"
```

### Principais Diferenças
- **executor assíncrono** via embassy-executor
- **timer assíncrono** via embassy-time  
- **I2C assíncrono** via embedded-hal-async
- **WiFi assíncrono** via esp-wifi com embassy-net
- **integração** via esp-hal-embassy

## Análise dos Exemplos Existentes

### 📚 Projeto simple-bme280-02 (Análise)
**Estrutura encontrada**:
- Driver BME280 customizado com embedded-hal 1.0
- Implementação I2C síncrona usando esp-hal
- Calibração básica (simplificada) dos sensores
- Interface modular com I2cDevice wrapper

**Pontos Chave**:
- Endereços I2C: 0x76 (primário), 0x77 (secundário)
- Registradores: Temperature(0xFA), Pressure(0xF7), Humidity(0xFD)
- Chip ID esperado: 0x60
- GPIO configurado: SDA=GPIO8, SCL=GPIO9

### 🔍 Pesquisa Embassy (GitHub + 2025)
**Projeto Referência**: `claudiomattera/esp32c3-embassy`
- ESP32-C3 + BME280 + Embassy + I2C async
- Dependências atualizadas para 2025
- Implementação completa com deep sleep
- WiFi time synchronization

**Dependências Embassy Validadas (2025)**:
- embassy-executor 0.7 + task-arena-size-20480
- embassy-time 0.4.0  
- esp-hal 0.23.1 + esp32c3 features
- esp-hal-embassy 0.6 + esp32c3 features
- embedded-hal-async 1.0

## Implementação BME280 + Embassy

### 🌡️ Módulo bme280-embassy (Implementado)
**Características**:
- **Base**: Template blinky migrado para Embassy
- **Async Tasks**: Sensor reading + LED heartbeat
- **I2C Async**: embedded-hal-async + bme280-rs crate
- **Hardware**: GPIO8(SDA), GPIO9(SCL), GPIO3(LED)
- **Timing**: Leituras a cada 2 segundos via embassy-time

**Arquitetura**:
```rust
#[embassy_executor::task]
async fn sensor_task() - Leitura contínua BME280
#[embassy_executor::task] 
async fn led_task() - LED heartbeat
#[esp_hal::main]
async fn main() - Spawner + setup
```

### Padrões de Código Estabelecidos
- **NO EMOJIS** em código de produção (apenas na documentação)
- **esp-hal + Embassy** como stack padrão (não esp-idf)
- **async/await** para todas operações I/O
- **embedded-hal-async** para abstração de hardware
- **Task separation** para responsabilidades distintas