#!/bin/bash

# ESP32-C3 Hardware Validation Script
# Validates hardware connections and system functionality

set -e

echo "🔍 ESP32-C3 Hardware Validation"
echo "==============================="

# Check USB connection
echo "1️⃣ Checking USB connection..."
if lsusb | grep -q "303a:1001"; then
    echo "✅ ESP32-C3 USB connection detected"
else
    echo "❌ ESP32-C3 not detected via USB"
    echo "   Please check:"
    echo "   - USB cable is properly connected"
    echo "   - ESP32-C3 is powered on"
    echo "   - USB drivers are installed"
    exit 1
fi

# Check serial port
echo ""
echo "2️⃣ Checking serial port access..."
ESP32_PORT=""
for port in /dev/ttyACM* /dev/ttyUSB* /dev/cu.usbmodem*; do
    if [ -e "$port" ] && [ -w "$port" ]; then
        ESP32_PORT="$port"
        echo "✅ Serial port found: $port"
        break
    fi
done

if [ -z "$ESP32_PORT" ]; then
    echo "❌ No writable serial port found"
    echo "   Please check:"
    echo "   - User has permission to access serial ports"
    echo "   - Run: sudo usermod -a -G dialout $USER"
    echo "   - Then logout and login again"
    exit 1
fi

# Check BME280 sensor connection (requires flashed firmware)
echo ""
echo "3️⃣ BME280 sensor validation..."
echo "   This will be validated after firmware deployment"
echo "   Expected I2C address: 0x76 or 0x77"

# Check WiFi network availability
echo ""
echo "4️⃣ Network connectivity check..."
WIFI_SSID="${WIFI_SSID:-YourWiFiNetwork}"
if command -v iwlist >/dev/null 2>&1; then
    if iwlist scan 2>/dev/null | grep -q "$WIFI_SSID"; then
        echo "✅ WiFi network '$WIFI_SSID' is available"
    else
        echo "⚠️  WiFi network '$WIFI_SSID' not detected"
        echo "   Please verify WiFi network name and availability"
    fi
else
    echo "ℹ️  WiFi scanning not available (iwlist not found)"
    echo "   Please verify WiFi network '$WIFI_SSID' is available"
fi

# Check MQTT broker connectivity
echo ""
echo "5️⃣ MQTT broker connectivity..."
MQTT_BROKER_IP="${MQTT_BROKER_IP:-192.168.1.100}"
if command -v nc >/dev/null 2>&1; then
    if nc -z "$MQTT_BROKER_IP" 1883 2>/dev/null; then
        echo "✅ MQTT broker reachable at $MQTT_BROKER_IP:1883"
    else
        echo "⚠️  MQTT broker not reachable at $MQTT_BROKER_IP:1883"
        echo "   Please verify:"
        echo "   - MQTT broker is running"
        echo "   - Network connectivity to broker"
        echo "   - Firewall allows port 1883"
    fi
else
    echo "ℹ️  Network connectivity check not available (nc not found)"
    echo "   Please verify MQTT broker at $MQTT_BROKER_IP:1883"
fi

# Check container runtime
echo ""
echo "6️⃣ Container runtime check..."
if command -v docker >/dev/null 2>&1; then
    if docker ps >/dev/null 2>&1; then
        echo "✅ Docker is available and running"
    else
        echo "⚠️  Docker found but not accessible"
        echo "   Please run: sudo usermod -a -G docker $USER"
        echo "   Then logout and login again"
    fi
elif command -v podman >/dev/null 2>&1; then
    echo "✅ Podman is available"
else
    echo "❌ No container runtime found (Docker or Podman required)"
    exit 1
fi

echo ""
echo "🎯 Hardware Validation Summary:"
echo "==============================="
echo "✅ ESP32-C3 USB connection"
echo "✅ Serial port access"
echo "ℹ️  BME280 sensor (validated post-deployment)"
echo "ℹ️  WiFi network: $WIFI_SSID"
echo "ℹ️  MQTT broker: $MQTT_BROKER_IP:1883"
echo "✅ Container runtime available"
echo ""
echo "🚀 Ready for deployment!"
echo "   Run: ./deploy.sh"