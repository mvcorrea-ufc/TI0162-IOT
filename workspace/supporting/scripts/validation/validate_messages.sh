#!/bin/bash
# ESP32-C3 IoT MQTT Message Format Validation Script
# 
# This script validates that all three applications produce consistent
# MQTT message formats according to the standardization specification.

set -euo pipefail

BROKER_IP="10.10.10.210"
VALIDATION_TIME=300  # 5 minutes
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${SCRIPT_DIR}/validation_messages.log"

echo "=== ESP32-C3 IoT Message Format Validation ==="
echo "Broker: ${BROKER_IP}"
echo "Validation Duration: ${VALIDATION_TIME} seconds"
echo "Log File: ${LOG_FILE}"
echo ""

# Check if mosquitto_sub is available
if ! command -v mosquitto_sub &> /dev/null; then
    echo "❌ ERROR: mosquitto_sub not found. Please install mosquitto clients."
    echo "   Ubuntu/Debian: sudo apt-get install mosquitto-clients"
    echo "   macOS: brew install mosquitto"
    exit 1
fi

# Check if jq is available for JSON validation
if ! command -v jq &> /dev/null; then
    echo "⚠️  WARNING: jq not found. JSON validation will be limited."
    echo "   Ubuntu/Debian: sudo apt-get install jq"
    echo "   macOS: brew install jq"
    VALIDATE_JSON=false
else
    VALIDATE_JSON=true
fi

# Initialize counters
SENSOR_SYNC_COUNT=0
SENSOR_ASYNC_COUNT=0
SENSOR_FULL_COUNT=0
HEARTBEAT_SYNC_COUNT=0
HEARTBEAT_ASYNC_COUNT=0
HEARTBEAT_FULL_COUNT=0
STATUS_SYNC_COUNT=0
STATUS_ASYNC_COUNT=0
STATUS_FULL_COUNT=0
VALID_FORMAT_COUNT=0
INVALID_FORMAT_COUNT=0

# Create log file
echo "# ESP32-C3 IoT Message Validation Log - $(date)" > "$LOG_FILE"
echo "# Format: [TIMESTAMP] TOPIC JSON_PAYLOAD VALIDATION_RESULT" >> "$LOG_FILE"

echo "🔍 Starting MQTT message monitoring..."
echo "📡 Subscribing to esp32c3/+/+/+ for ${VALIDATION_TIME} seconds..."
echo ""

# Function to validate JSON message format
validate_json_format() {
    local json="$1"
    local topic="$2"
    local timestamp="$3"
    
    if [ "$VALIDATE_JSON" = true ]; then
        # Check for required fields based on message type
        if [[ "$topic" == *"/sensor/"* ]]; then
            # Sensor message validation
            if echo "$json" | jq -e '.temperature and .humidity and .pressure and .timestamp and .device_id and .reading_count' >/dev/null 2>&1; then
                local device_id=$(echo "$json" | jq -r '.device_id')
                local temp=$(echo "$json" | jq -r '.temperature')
                local humidity=$(echo "$json" | jq -r '.humidity')
                local pressure=$(echo "$json" | jq -r '.pressure')
                local count=$(echo "$json" | jq -r '.reading_count')
                
                echo "  ✅ SENSOR Valid: device=$device_id temp=${temp}°C hum=${humidity}% pres=${pressure}hPa count=$count"
                echo "[$timestamp] $topic $json VALID_SENSOR" >> "$LOG_FILE"
                ((VALID_FORMAT_COUNT++))
                return 0
            else
                echo "  ❌ SENSOR Invalid: Missing required fields (temperature, humidity, pressure, timestamp, device_id, reading_count)"
                echo "[$timestamp] $topic $json INVALID_SENSOR" >> "$LOG_FILE"
                ((INVALID_FORMAT_COUNT++))
                return 1
            fi
        elif [[ "$topic" == *"/heartbeat/"* ]]; then
            # Heartbeat message validation
            if echo "$json" | jq -e '.status and .timestamp and .device_id and .sequence' >/dev/null 2>&1; then
                local device_id=$(echo "$json" | jq -r '.device_id')
                local status=$(echo "$json" | jq -r '.status')
                local sequence=$(echo "$json" | jq -r '.sequence')
                
                echo "  ✅ HEARTBEAT Valid: device=$device_id status=$status sequence=$sequence"
                echo "[$timestamp] $topic $json VALID_HEARTBEAT" >> "$LOG_FILE"
                ((VALID_FORMAT_COUNT++))
                return 0
            else
                # Check for legacy format (simple "ping")
                if [[ "$json" == "ping" ]] || [[ "$json" == '"ping"' ]]; then
                    echo "  ⚠️  HEARTBEAT Legacy: Simple ping format (needs migration)"
                    echo "[$timestamp] $topic $json LEGACY_HEARTBEAT" >> "$LOG_FILE"
                    ((VALID_FORMAT_COUNT++))
                    return 0
                else
                    echo "  ❌ HEARTBEAT Invalid: Missing required fields (status, timestamp, device_id, sequence)"
                    echo "[$timestamp] $topic $json INVALID_HEARTBEAT" >> "$LOG_FILE"
                    ((INVALID_FORMAT_COUNT++))
                    return 1
                fi
            fi
        elif [[ "$topic" == *"/status/"* ]]; then
            # Status message validation
            if echo "$json" | jq -e '.status and .uptime_seconds and .free_heap_bytes and .wifi_rssi_dbm and .sensor_readings and .timestamp and .device_id and .architecture' >/dev/null 2>&1; then
                local device_id=$(echo "$json" | jq -r '.device_id')
                local status=$(echo "$json" | jq -r '.status')
                local uptime=$(echo "$json" | jq -r '.uptime_seconds')
                local arch=$(echo "$json" | jq -r '.architecture')
                
                echo "  ✅ STATUS Valid: device=$device_id status=$status uptime=${uptime}s arch=$arch"
                echo "[$timestamp] $topic $json VALID_STATUS" >> "$LOG_FILE"
                ((VALID_FORMAT_COUNT++))
                return 0
            else
                echo "  ❌ STATUS Invalid: Missing required fields"
                echo "[$timestamp] $topic $json INVALID_STATUS" >> "$LOG_FILE"
                ((INVALID_FORMAT_COUNT++))
                return 1
            fi
        fi
    else
        # Basic validation without jq
        if [[ "$json" == *"temperature"* && "$json" == *"humidity"* && "$json" == *"pressure"* ]]; then
            echo "  ✅ Basic validation passed (temperature, humidity, pressure found)"
            echo "[$timestamp] $topic $json BASIC_VALID" >> "$LOG_FILE"
            ((VALID_FORMAT_COUNT++))
            return 0
        else
            echo "  ❌ Basic validation failed"
            echo "[$timestamp] $topic $json BASIC_INVALID" >> "$LOG_FILE"
            ((INVALID_FORMAT_COUNT++))
            return 1
        fi
    fi
}

# Start monitoring with timeout
timeout "$VALIDATION_TIME" mosquitto_sub -h "$BROKER_IP" -t "esp32c3/+/+/+" -v 2>/dev/null | while IFS= read -r line; do
    timestamp=$(date '+%H:%M:%S')
    
    # Parse topic and payload
    topic=$(echo "$line" | cut -d' ' -f1)
    payload=$(echo "$line" | cut -d' ' -f2-)
    
    echo "[$timestamp] $topic"
    echo "  📄 Payload: $payload"
    
    # Count messages by type and architecture
    if [[ "$topic" == *"/sensor/"* ]]; then
        if [[ "$topic" == *"/sync" ]]; then
            ((SENSOR_SYNC_COUNT++))
        elif [[ "$topic" == *"/async" ]]; then
            ((SENSOR_ASYNC_COUNT++))
        elif [[ "$topic" == *"/full" ]]; then
            ((SENSOR_FULL_COUNT++))
        fi
    elif [[ "$topic" == *"/heartbeat/"* ]]; then
        if [[ "$topic" == *"/sync" ]]; then
            ((HEARTBEAT_SYNC_COUNT++))
        elif [[ "$topic" == *"/async" ]]; then
            ((HEARTBEAT_ASYNC_COUNT++))
        elif [[ "$topic" == *"/full" ]]; then
            ((HEARTBEAT_FULL_COUNT++))
        fi
    elif [[ "$topic" == *"/status/"* ]]; then
        if [[ "$topic" == *"/sync" ]]; then
            ((STATUS_SYNC_COUNT++))
        elif [[ "$topic" == *"/async" ]]; then
            ((STATUS_ASYNC_COUNT++))
        elif [[ "$topic" == *"/full" ]]; then
            ((STATUS_FULL_COUNT++))
        fi
    fi
    
    # Validate message format
    validate_json_format "$payload" "$topic" "$timestamp"
    
    echo ""
done

echo ""
echo "=== Validation Summary ==="
echo ""

# Message counts by architecture
echo "📊 Message Counts by Architecture:"
echo "   main-nodeps (sync):  sensor=$SENSOR_SYNC_COUNT heartbeat=$HEARTBEAT_SYNC_COUNT status=$STATUS_SYNC_COUNT"
echo "   main-min (async):    sensor=$SENSOR_ASYNC_COUNT heartbeat=$HEARTBEAT_ASYNC_COUNT status=$STATUS_ASYNC_COUNT"  
echo "   main-app (full):     sensor=$SENSOR_FULL_COUNT heartbeat=$HEARTBEAT_FULL_COUNT status=$STATUS_FULL_COUNT"
echo ""

# Total counts
total_messages=$((VALID_FORMAT_COUNT + INVALID_FORMAT_COUNT))
echo "📈 Format Validation Results:"
echo "   Valid Messages:      $VALID_FORMAT_COUNT"
echo "   Invalid Messages:    $INVALID_FORMAT_COUNT"
echo "   Total Messages:      $total_messages"

if [ "$total_messages" -gt 0 ]; then
    success_rate=$(( (VALID_FORMAT_COUNT * 100) / total_messages ))
    echo "   Success Rate:        ${success_rate}%"
else
    echo "   Success Rate:        N/A (no messages received)"
fi

echo ""

# Final assessment
if [ "$INVALID_FORMAT_COUNT" -eq 0 ] && [ "$VALID_FORMAT_COUNT" -gt 0 ]; then
    echo "🎉 SUCCESS: All received messages passed format validation!"
    echo "✅ Message format standardization is working correctly."
elif [ "$total_messages" -eq 0 ]; then
    echo "⚠️  WARNING: No messages received during validation period."
    echo "🔧 Check that ESP32-C3 devices are running and connected to broker $BROKER_IP"
else
    echo "❌ FAILURE: Some messages failed format validation."
    echo "🔧 Review log file: $LOG_FILE"
    echo "🔧 Apply standardization changes from IMPLEMENTATION_CHANGES_GUIDE.md"
fi

echo ""
echo "📋 Detailed log available at: $LOG_FILE"
echo "🔄 To run timing validation: ./validate_timing.sh"
echo ""