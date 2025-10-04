#!/bin/bash

# ESP32-C3 IoT System Deployment Script
# Deploys the Phase 2 architecture to real ESP32-C3 hardware

set -e

echo "🚀 ESP32-C3 IoT System Deployment"
echo "================================="

# Configuration
WIFI_SSID="${WIFI_SSID:-YourWiFiNetwork}"
WIFI_PASSWORD="${WIFI_PASSWORD:-YourWiFiPassword}"
MQTT_BROKER_IP="${MQTT_BROKER_IP:-192.168.1.100}"

# Check hardware connection
echo "📡 Checking ESP32-C3 connection..."
if lsusb | grep -q "303a:1001"; then
    echo "✅ ESP32-C3 detected via USB"
else
    echo "❌ ESP32-C3 not detected. Please check USB connection."
    echo "   Expected: USB device 303a:1001 (Espressif ESP32-C3)"
    exit 1
fi

# Find ESP32-C3 serial port
ESP32_PORT=""
for port in /dev/ttyACM* /dev/ttyUSB* /dev/cu.usbmodem*; do
    if [ -e "$port" ]; then
        ESP32_PORT="$port"
        break
    fi
done

if [ -z "$ESP32_PORT" ]; then
    echo "❌ ESP32-C3 serial port not found"
    exit 1
fi

echo "✅ ESP32-C3 found on port: $ESP32_PORT"

# Set up environment variables
export WIFI_SSID="$WIFI_SSID"
export WIFI_PASSWORD="$WIFI_PASSWORD" 
export MQTT_BROKER_IP="$MQTT_BROKER_IP"
export MQTT_BROKER_PORT="1883"
export MQTT_CLIENT_ID="esp32c3-iot-$(date +%s)"
export MQTT_TOPIC_PREFIX="iot/sensors"

echo "🔧 Configuration:"
echo "   WiFi SSID: $WIFI_SSID"
echo "   MQTT Broker: $MQTT_BROKER_IP:$MQTT_BROKER_PORT"
echo "   Client ID: $MQTT_CLIENT_ID"

# Build the application
echo ""
echo "🔨 Building ESP32-C3 application (Phase 2 dependency injection architecture)..."
source /root/export-esp.sh 2>/dev/null || true
cargo build --release --bin main_container

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"

# Flash to ESP32-C3
echo ""
echo "📤 Flashing to ESP32-C3..."
espflash flash target/riscv32imc-unknown-none-elf/release/main_container \
    --port "$ESP32_PORT" \
    --baud 460800

if [ $? -ne 0 ]; then
    echo "❌ Flashing failed"
    exit 1
fi

echo "✅ Flashing successful"

# Monitor serial output
echo ""
echo "📺 Starting serial monitor..."
echo "   Press Ctrl+C to stop monitoring"
echo "   ESP32-C3 console available at $ESP32_PORT"
echo ""

# Use espflash monitor for better integration
espflash monitor --port "$ESP32_PORT" --baud 115200