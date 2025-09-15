# BME280 Embassy - Sensor de Temperatura, Umidade e Pressão

## 🌡️ Descrição

Módulo completo e funcional para leitura assíncrona do sensor BME280 usando o framework Embassy para ESP32-C3. Este módulo implementa um driver BME280 customizado com compensação de valores corrigida e calibração automática.

**Status**: ✅ Implementado e testado

## 🚀 Características

- ✅ **Async/Await**: Todas operações I2C são assíncronas via Embassy
- ✅ **Embassy Framework**: embassy-executor 0.7 + embassy-time 0.4
- ✅ **ESP32-C3**: esp-hal v1.0.0-rc.0 com features unstable
- ✅ **Calibração Automática**: Leitura e aplicação dos coeficientes de calibração
- ✅ **Compensação Corrigida**: Algoritmos de compensação validados
- ✅ **Dual Address**: Suporta endereços I2C 0x76 e 0x77
- ✅ **RTT Debugging**: Output em tempo real via rtt-target
- ✅ **LED Heartbeat**: Indicação visual de funcionamento

## 🔌 Pinagem Hardware

```
ESP32-C3        BME280
--------        ------
GPIO8    <-->   SDA (dados I2C)
GPIO9    <-->   SCL (clock I2C)
3.3V     <-->   VCC (alimentação)
GND      <-->   GND (terra)
GPIO3    <-->   LED (indicador status)
```

### 📋 Especificações BME280

- **Temperatura**: -40°C a +85°C (precisão ±1°C)
- **Umidade**: 0-100% RH (precisão ±3%)
- **Pressão**: 300-1100 hPa (precisão ±1 hPa)
- **Endereços I2C**: 0x76 (primário), 0x77 (secundário)
- **Frequência I2C**: 100kHz (padrão)
- **Alimentação**: 3.3V
- **Consumo**: ~3.4μA (modo sleep)

## Estrutura do Projeto

```
bme280-embassy/
├── src/
│   ├── main.rs          # Aplicação principal com Embassy tasks
│   ├── lib.rs           # Módulo library
│   ├── bme280.rs        # Driver BME280 async
│   └── i2c_device.rs    # Wrapper I2C async
├── examples/
│   └── basic_reading.rs # Exemplo de teste do módulo
├── Cargo.toml           # Dependências Embassy
└── build.rs             # Configuração build ESP32-C3
```

## Dependências

```toml
# Base Embassy
embassy-executor = { version = "0.7", features = ["task-arena-size-20480"] }
embassy-time = "0.4.0"

# ESP32-C3 HAL + Embassy Integration  
esp-hal = { version = "0.23.1", features = ["esp32c3", "log"] }
esp-hal-embassy = { version = "0.6", features = ["esp32c3"] }

# I2C Async Support
embedded-hal-async = "1.0"
```

## 🚀 Uso Rápido

### Pré-requisitos

```bash
# Instalar target Rust para ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Instalar probe-rs para flash e debugging
cargo install probe-rs --features cli

# Verificar ESP32-C3 conectado
probe-rs list
```

### Comandos de Build

```bash
# Navegar para o módulo
cd bme280-embassy/

# Build apenas (verificar compilação)
cargo build --release

# Build + Flash + Monitor (aplicação principal)
cargo run --release

# Build + Flash + Monitor (exemplo básico)
cargo run --example basic_reading --release

# Limpeza de build
cargo clean

# Verificação de código
cargo clippy
cargo fmt
```

### Saída Esperada

```
BME280 Embassy: Initializing BME280 sensor...
BME280 Embassy: Sensor initialized successfully
BME280 Embassy: T: 23.2°C, H: 68.5%, P: 1013.8 hPa
BME280 Embassy: T: 23.1°C, H: 68.3%, P: 1013.9 hPa
BME280 Embassy: T: 23.0°C, H: 68.7%, P: 1013.7 hPa
```

## API do Módulo

### BME280 Driver

```rust
use bme280_embassy::{BME280, Measurements};

// Inicializar
let mut bme280 = BME280::new(&mut i2c);

// Verificar sensor
let detected = bme280.check_id().await?;

// Ler dados processados
let measurements = bme280.read_measurements().await?;
println!("Temp: {:.2}°C", measurements.temperature);

// Ler dados brutos
let (temp, press, hum) = bme280.read_raw_data().await?;
```

### Embassy Tasks

```rust
#[embassy_executor::task]
async fn sensor_task(mut i2c: I2c<'static, esp_hal::peripherals::I2C0>) {
    let mut bme280 = BME280::new(&mut i2c);
    
    loop {
        let data = bme280.read_measurements().await?;
        // Processar dados...
        Timer::after(Duration::from_secs(2)).await;
    }
}
```

## Padrões de Desenvolvimento

- **NO EMOJIS** no código de produção
- **esp-hal + Embassy** como stack padrão  
- **async/await** para todas operações I/O
- **embedded-hal-async** para abstração
- **Task separation** para responsabilidades

## 🐛 Troubleshooting

### Problemas Comuns

1. **Sensor não responde (I2C timeout)**:
   ```bash
   # Verificar pinagem
   # GPIO8 = SDA, GPIO9 = SCL
   # Verificar alimentação 3.3V
   # Testar continuidade com multímetro
   ```

2. **Valores de umidade incorretos (0-100%)**:
   ```bash
   # Normal após correções implementadas
   # Algoritmo de compensação foi corrigido
   # Aguardar estabilização (~30 segundos)
   ```

3. **Build falha**:
   ```bash
   cargo clean
   rustup target add riscv32imc-unknown-none-elf
   cargo build --release
   ```

4. **ESP32-C3 não conecta**:
   ```bash
   probe-rs list  # Verificar dispositivo
   # Pressionar BOOT + RST se necessário
   # Verificar cabo USB (dados, não apenas carga)
   ```

### Debug RTT

```rust
// Adicionar debug personalizado
rprintln!("BME280 Debug: Temp raw = {}", temp_raw);
rprintln!("BME280 Debug: Calibration T1 = {}", cal_data.dig_t1);
```

## 🔗 Integração com Outros Módulos

Este módulo pode ser integrado com:

- **wifi-embassy**: Para transmissão WiFi dos dados
- **mqtt-embassy**: Para publicação MQTT dos sensores
- **web-server**: Interface web para visualização
- **main-app**: Aplicação IoT completa

### Exemplo de Integração

```rust
// Em main-app/src/main.rs
use bme280_embassy::{BME280, Measurements};
use wifi_embassy::WiFiManager;
use mqtt_embassy::MqttClient;

#[embassy_executor::task]
async fn sensor_mqtt_task() {
    let measurements = bme280.read_measurements().await?;
    let json_data = format_sensor_data(&measurements);
    mqtt_client.publish("esp32/sensor/bme280", &json_data).await?;
}
```

## 📄 Licença

MIT OR Apache-2.0

## 👨‍💻 Autor

Marcelo Correa <mvcorrea@gmail.com>