# TI0162 - Internet das Coisas - Projeto IoT Completo

Este workspace contém um projeto IoT completo e funcional desenvolvido em Rust para ESP32-C3 usando o framework Embassy. O sistema implementa coleta de dados ambientais via sensor BME280, conectividade WiFi e transmissão MQTT, formando um pipeline IoT robusto e modular.

**Status do Projeto**: ✅ Sistema IoT totalmente funcional e operacional

## 🏗️ Arquitetura Modular Implementada

### ✅ bme280-embassy/ - Sensor BME280 + Embassy
**Status**: Implementado e testado  
**Função**: Leitura assíncrona de temperatura, umidade e pressão  
**Tecnologia**: Embassy async + I2C + BME280 customizado  
**Hardware**: GPIO8(SDA), GPIO9(SCL), GPIO3(LED)  
**Saída**: RTT debugging com valores compensados

### ✅ wifi-embassy/ - Conectividade WiFi
**Status**: Implementado e testado  
**Função**: Conexão WiFi robusta com reconexão automática  
**Tecnologia**: Embassy + esp-wifi + DHCP  
**IP Testado**: 10.10.10.214  
**Features**: Network stack completo para TCP/UDP

### ✅ mqtt-embassy/ - Cliente MQTT
**Status**: Implementado e testado  
**Função**: Publicação MQTT assíncrona via TCP sockets  
**Tecnologia**: Embassy + protocolo MQTT 3.1.1  
**Broker Testado**: 10.10.10.210:1883  
**Mensagens**: JSON estruturado para sensores, status e heartbeat

### ✅ Sistema Integrado - Pipeline IoT Completo
**Status**: Operacional e validado  
**Fluxo**: ESP32-C3 → BME280 → WiFi → MQTT → Mosquitto → Subscribers  
**Exemplo**: wifi-embassy/examples/wifi_mqtt_test.rs  
**Periodicidade**: 30s sensor, 2.5min heartbeat, 5min status

## 🚀 Início Rápido - Sistema IoT Completo

### Pré-requisitos

```bash
# Instalar Rust + target ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Instalar probe-rs
cargo install probe-rs --features cli

# Verificar ESP32-C3 conectado
probe-rs list
```

### Configuração das Credenciais

Cada módulo possui `.cargo/config.toml` para configuração via variáveis de ambiente:

```toml
# Exemplo: wifi-embassy/.cargo/config.toml
[env]
WIFI_SSID = "SuaRedeWiFi"
WIFI_PASSWORD = "SuaSenhaWiFi"
MQTT_BROKER_IP = "192.168.1.100"
MQTT_BROKER_PORT = "1883"
```

### Teste do Sistema Completo

```bash
# 1. Testar sensor BME280
cd bme280-embassy/
cargo run --release

# 2. Testar conectividade WiFi
cd ../wifi-embassy/
cargo run --example wifi_test_new --release

# 3. Configurar broker MQTT
sudo apt install mosquitto mosquitto-clients
sudo systemctl start mosquitto

# 4. Monitor MQTT (terminal separado)
mosquitto_sub -h [SEU_IP] -p 1883 -t "esp32/#" -v

# 5. Sistema IoT completo
cargo run --example wifi_mqtt_test --release
```

## 📊 Dados Publicados no MQTT

### Sensor BME280 (esp32/sensor/bme280)
```json
{
  "temperature": 23.2,
  "humidity": 68.5,
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

## 📂 Estrutura de Arquivos

```
workspace/
├── bme280-embassy/          # 🌡️ Sensor de temperatura/umidade/pressão
│   ├── src/                 # Driver BME280 customizado + Embassy
│   ├── examples/            # Exemplos de leitura
│   └── README.md           # Documentação detalhada
├── wifi-embassy/            # 📡 Conectividade WiFi robusta
│   ├── src/                 # WiFi manager + Embassy network stack
│   ├── examples/            # Testes WiFi + integração MQTT
│   └── README.md           # Documentação detalhada
├── mqtt-embassy/            # 📨 Cliente MQTT assíncrono
│   ├── src/                 # Cliente MQTT + estruturas JSON
│   ├── examples/            # Testes MQTT
│   └── README.md           # Documentação detalhada
├── examples/                # 📚 Projetos de referência externos
├── blinky/                 # 🏗️ Template base (esp-hal básico)
├── CLAUDE.md               # 📖 Documentação completa do projeto
├── .gitignore              # Exclusões git (target/, logs, etc.)
└── README.md               # Esta documentação
```

## 🛠️ Tecnologias e Dependências

### Stack Tecnológico Principal
- **Linguagem**: Rust (stable)
- **Target**: riscv32imc-unknown-none-elf (ESP32-C3)
- **Framework Async**: Embassy (executor 0.7 + time 0.4)
- **HAL**: esp-hal 1.0.0-rc.0 (ESP32-C3 unstable features)
- **WiFi**: esp-wifi 0.15.0 + smoltcp network stack
- **Debugging**: RTT (Real-Time Transfer) via rtt-target

### Dependências por Módulo

#### BME280 Embassy
```toml
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embedded-hal-async = "1.0"
```

#### WiFi Embassy
```toml
esp-wifi = { version = "0.15.0", features = ["esp32c3", "wifi", "smoltcp"] }
embassy-net = { version = "0.7", features = ["tcp", "udp", "dhcpv4"] }
esp-alloc = { version = "0.8.0" }
static_cell = "2.0"
```

#### MQTT Embassy
```toml
wifi-embassy = { path = "../wifi-embassy" }
embedded-io-async = "0.6"
serde = { version = "1.0", default-features = false }
serde-json-core = "0.6"
heapless = "0.8"
```

## 📋 Requisitos de Hardware

### ESP32-C3 DevKit
- **Microcontrolador**: ESP32-C3 (RISC-V single-core 160MHz)
- **Conectividade**: WiFi 2.4GHz (não suporta 5GHz)
- **GPIO**: 22 pinos digitais disponíveis
- **I2C**: GPIO8(SDA), GPIO9(SCL) para sensor BME280
- **Alimentação**: 3.3V via USB ou fonte externa
- **Flash**: 4MB mínimo recomendado

### Sensor BME280 (Opcional)
- **Interface**: I2C (endereço 0x76 ou 0x77)
- **Medições**: Temperatura (-40°C a +85°C), Umidade (0-100% RH), Pressão (300-1100 hPa)
- **Precisão**: ±1°C (temp), ±3% (umidade), ±1 hPa (pressão)
- **Alimentação**: 3.3V, ~3.4μA modo sleep

### Infraestrutura de Rede
- **WiFi**: Rede 2.4GHz com DHCP habilitado
- **MQTT Broker**: Mosquitto ou similar (testado: 10.10.10.210:1883)
- **Monitoramento**: Cliente mosquitto_sub para visualizar mensagens

### Ambiente de Desenvolvimento
- **SO**: Linux/macOS/Windows com suporte USB
- **Rust**: stable toolchain + target riscv32imc-unknown-none-elf
- **Debugging**: probe-rs para flash e RTT
- **USB**: Cabo de dados (não apenas carregamento)

## 🔧 Comandos de Desenvolvimento

### Build e Flash (Todos os Módulos)
```bash
# Build debug (compilação mais rápida)
cargo build

# Build release (otimizado, recomendado para ESP32)
cargo build --release

# Flash + monitor RTT (aplicação principal)
cargo run --release

# Flash + monitor RTT (exemplo específico)
cargo run --example [NOME_EXEMPLO] --release

# Limpeza de artefatos
cargo clean

# Verificação de código
cargo clippy
cargo fmt
```

### Comandos Específicos por Módulo

#### BME280 Embassy
```bash
cd bme280-embassy/
cargo run --release                         # App principal
cargo run --example basic_reading --release # Teste básico
```

#### WiFi Embassy
```bash
cd wifi-embassy/
cargo run --example wifi_test --release      # Teste básico WiFi
cargo run --example wifi_test_new --release  # Teste detalhado
cargo run --example wifi_mqtt_test --release # Sistema completo
```

#### MQTT Embassy
```bash
cd mqtt-embassy/
cargo run --example mqtt_test_working --features examples --release
```

## 🔗 Padrões de Integração Implementados

### Sistema IoT Completo
O projeto demonstra integração completa entre todos os módulos:

```rust
// Exemplo funcional em wifi-embassy/examples/wifi_mqtt_test.rs

// 1. Inicializar Embassy + WiFi
let wifi_manager = WiFiManager::new(spawner, /* ... */).await?;
let stack = wifi_manager.get_stack();

// 2. Criar dados do sensor (mock ou real BME280)
let temperature = 23.5;
let humidity = 68.2;
let pressure = 1013.8;

// 3. Conectar ao broker MQTT
let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
socket.connect(("10.10.10.210".parse().unwrap(), 1883)).await?;

// 4. Publicar dados via MQTT
let json_payload = format!(
    r#"{{"temperature":{:.1},"humidity":{:.1},"pressure":{:.1}}}"#,
    temperature, humidity, pressure
);
socket.write_all(&mqtt_publish_packet).await?;
```

### Padrões de Código Estabelecidos

#### Embassy Async Tasks
```rust
#[embassy_executor::task]
async fn sensor_task() {
    loop {
        let data = sensor.read().await;
        rprintln!("Sensor: {:?}", data);
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[embassy_executor::task]
async fn mqtt_task(wifi_manager: &'static WiFiManager) {
    // Publicação MQTT periódica
}
```

#### Configuração via Ambiente
```rust
const WIFI_SSID: &str = env!("WIFI_SSID", "Configure em .cargo/config.toml");
const MQTT_BROKER: &str = env!("MQTT_BROKER_IP", "192.168.1.100");
```

#### Error Handling Robusto
```rust
match socket.connect(broker_addr).await {
    Ok(()) => rprintln!("✅ Conectado ao broker"),
    Err(e) => {
        rprintln!("❌ Falha na conexão: {:?}", e);
        return; // Retry no próximo ciclo
    }
}
```

## 🐛 Troubleshooting

### Problemas Comuns e Soluções

#### Hardware e Conectividade
1. **ESP32-C3 não conecta**:
   ```bash
   probe-rs list  # Deve mostrar o dispositivo
   # Se não aparecer: verificar cabo USB (dados), pressionar BOOT+RST
   ```

2. **WiFi não conecta**:
   ```bash
   # Verificar credenciais em .cargo/config.toml
   # Confirmar rede 2.4GHz (ESP32-C3 não suporta 5GHz)
   # Testar SSID case-sensitive
   ```

3. **DHCP falha**:
   ```bash
   # Verificar router funcionando
   # Confirmar pool DHCP disponível
   # Testar conectividade com outro dispositivo
   ```

#### Desenvolvimento e Build
4. **Build falha**:
   ```bash
   cargo clean
   rustup target add riscv32imc-unknown-none-elf
   cargo build --release
   ```

5. **Embassy time driver erro**:
   ```bash
   # Erro: schedule_wake called before esp_hal_embassy::init()
   # Solução: Chamar esp_hal_embassy::init() antes de WiFiManager::new()
   ```

#### MQTT e Rede
6. **MQTT broker inacessível**:
   ```bash
   ping 10.10.10.210
   telnet 10.10.10.210 1883
   sudo systemctl status mosquitto
   ```

7. **Mensagens MQTT não aparecem**:
   ```bash
   # Verificar tópicos: mosquitto_sub -h [BROKER] -t "#" -v
   # Debug packet format no código ESP32
   ```

### Estratégias de Debug

#### RTT Debugging
```rust
// Adicionar debug detalhado
rprintln!("WiFi Status: {:?}", wifi_status);
rprintln!("IP Config: {:?}", stack.config_v4());
rprintln!("MQTT Packet: {:02X?}", &packet[..20]);
```

#### Teste Modular
```bash
# 1. Verificar hardware básico
cd blinky/ && cargo run --release

# 2. Testar sensor (se disponível)
cd ../bme280-embassy/ && cargo run --release

# 3. Testar WiFi isoladamente
cd ../wifi-embassy/ && cargo run --example wifi_test_new --release

# 4. Sistema completo
cargo run --example wifi_mqtt_test --release
```

#### Monitor de Rede
```bash
# Terminal 1: Monitor MQTT
mosquitto_sub -h 10.10.10.210 -p 1883 -t "esp32/#" -v

# Terminal 2: Executar ESP32
cd wifi-embassy/
cargo run --example wifi_mqtt_test --release

# Terminal 3: Monitor de conectividade
ping 10.10.10.214  # IP do ESP32
```

## 📈 Performance e Otimização

### Build e Runtime
- **Release obrigatório**: Sempre usar `--release` para ESP32-C3 (builds debug são muito lentos)
- **Heap allocation**: 72KB configurado para WiFi operations
- **RTT minimal overhead**: Debugging RTT tem impacto mínimo na performance
- **Network stack**: Operations são síncronas por design para compatibilidade MQTT

### Timing do Sistema IoT
- **Sensor data**: Publicação a cada 30 segundos
- **Heartbeat**: A cada 2.5 minutos (5 ciclos)
- **Device status**: A cada 5 minutos (10 ciclos)
- **WiFi reconnect**: Automático em caso de desconexão
- **MQTT reconnect**: Nova conexão TCP a cada ciclo (robusto)

## 🔮 Expansão Futura

### Módulos Planejados
- **web-server**: Interface web para monitoramento em tempo real
- **main-app**: Aplicação final integrando todos os módulos
- **sensor-advanced**: Múltiplos sensores I2C/SPI
- **ble-simple**: Conectividade Bluetooth Low Energy como backup

### Melhorias Potenciais
- **Persistent MQTT**: Conexões MQTT persistentes (vs. reconnect a cada ciclo)
- **Deep Sleep**: Economia de energia entre leituras
- **OTA Updates**: Atualizações over-the-air
- **Data buffering**: Buffer local para casos de desconexão temporária
- **Time sync**: Sincronização de tempo via NTP
- **TLS/SSL**: Conexões seguras MQTT

## 🎯 Status do Projeto

### ✅ Implementado e Testado
- [x] Sensor BME280 com compensação corrigida
- [x] WiFi connectivity robusta com DHCP
- [x] Cliente MQTT completo com JSON
- [x] Pipeline IoT end-to-end funcional
- [x] Documentação completa de todos os módulos
- [x] Exemplos funcionais para cada componente

### 📊 Resultados Validados
- **Hardware**: ESP32-C3 DevKit funcionando perfeitamente
- **Sensor**: BME280 com leituras precisas (T: 23°C, H: 68%, P: 1013hPa)
- **WiFi**: Conexão estável com IP 10.10.10.214
- **MQTT**: Mensagens entregues com sucesso ao broker 10.10.10.210:1883
- **Subscribers**: mosquitto_sub recebendo dados JSON estruturados

### 🏆 Objetivos Alcançados
1. **Modularidade**: Cada componente funciona independentemente
2. **Robustez**: Sistema resiliente a desconexões e falhas
3. **Escalabilidade**: Arquitetura preparada para expansão
4. **Documentação**: READMEs detalhados em cada módulo
5. **Testabilidade**: Exemplos funcionais para validação

## 📄 Licença

MIT OR Apache-2.0

## 👨‍💻 Autor

Marcelo Correa <mvcorrea@gmail.com>

**Projeto TI0162 - Internet das Coisas**  
**Sistema IoT Completo com ESP32-C3 + Rust + Embassy**