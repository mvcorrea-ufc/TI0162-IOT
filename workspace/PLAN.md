# TI0162 - Internet das Coisas - Plano de Execução

## Status do Projeto: 🚀 Em Desenvolvimento

## Tasks Concluídas ✅

1. **Setup Inicial do Projeto**
   - Clone do repositório base rust-esp32-tmpl
   - Definição do projeto `blinky` como base de implementação
   - Criação da estrutura de documentação (CLAUDE.md, PLAN.md)
   - Definição da arquitetura modular baseada em blinky

## Tasks em Andamento 🔄

*Nenhuma task em andamento no momento*

## Tasks Pendentes 📋

2. **Módulo BME280 - Implementação**
   - Criar diretório `bme280-module/` baseado na estrutura do `blinky`
   - Copiar configuração base (Cargo.toml, build.rs) do projeto blinky
   - Implementar driver BME280 usando Embassy sobre esp-hal
   - Configurar I2C para comunicação com sensor
   - Implementar leitura de temperatura, umidade e pressão
   - Integrar com sistema RTT para debugging
   - Criar estruturas de dados para os valores do sensor

3. **Módulo BME280 - Validação**
   - Criar aplicação de teste para BME280
   - Implementar saída dos valores no console
   - Verificar precisão das leituras
   - Documentar interface do módulo

4. **Módulo WiFi - Implementação**
   - Criar diretório `wifi-module/`
   - Implementar conexão WiFi usando Embassy
   - Configurar conexão a access point local
   - Implementar gestão de reconexão automática

5. **Módulo WiFi - Validação**
   - Verificar aquisição de endereço IP via DHCP
   - Implementar teste de ping para validar conectividade
   - Criar logs de status da conexão
   - Documentar configuração de rede

6. **Servidor Web - Implementação**
   - Criar diretório `web-server/`
   - Implementar servidor HTTP básico
   - Criar página HTML para exibição dos dados BME280
   - Integrar dados do sensor com interface web

7. **Módulo MQTT - Implementação**
   - Criar diretório `mqtt-module/`
   - Implementar cliente MQTT usando Embassy
   - Configurar conexão ao broker Mosquitto
   - Implementar serialização JSON dos dados

8. **Módulo MQTT - Validação**
   - Configurar envio automático a cada 10 segundos
   - Testar conectividade com broker
   - Validar formato dos dados enviados
   - Implementar handling de erros de conexão

## Próximas Expansões 🔮

- Implementação de novos sensores
- Dashboard web avançado
- Armazenamento local de dados
- OTA updates
- Modos de baixo consumo

## Notas de Desenvolvimento 📝

**Base de Implementação - Projeto `blinky`:**
- Utilizar esp-hal v0.23.1 como HAL base
- Manter estrutura RTT para debugging (rprintln!)
- Copiar configurações base (Cargo.toml, build.rs)
- Preservar inicialização de periféricos do esp-hal

**Desenvolvimento Modular:**
- Cada módulo deve ser independente e reutilizável
- Usar async/await extensively com Embassy sobre esp-hal
- Implementar error handling robusto
- Manter logging estruturado via RTT para debugging
- Seguir convenções de código Rust

**Configuração Padrão Herdada:**
```toml
[dependencies]
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
esp-rom-sys = { version = "0.1", features = ["esp32c3"] }
defmt = "0.3"
rtt-target = "0.5"  
panic-rtt-target = "0.1"
```

---

**Última atualização**: 2025-09-12  
**Próxima revisão**: Após conclusão do módulo BME280