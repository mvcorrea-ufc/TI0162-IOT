#!/bin/bash
# ESP32-C3 IoT MQTT Timing Validation Script
#
# This script validates that all three applications follow the standardized
# timing intervals: 30s sensor / 60s heartbeat / 120s status

set -euo pipefail

BROKER_IP="10.10.10.210"
VALIDATION_TIME=600  # 10 minutes for proper timing analysis
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${SCRIPT_DIR}/validation_timing.log"

# Expected intervals (seconds)
EXPECTED_SENSOR_INTERVAL=30
EXPECTED_HEARTBEAT_INTERVAL=60
EXPECTED_STATUS_INTERVAL=120

# Tolerance (seconds)
TOLERANCE=5

echo "=== ESP32-C3 IoT Timing Validation ==="
echo "Broker: ${BROKER_IP}"
echo "Validation Duration: ${VALIDATION_TIME} seconds"
echo "Expected Intervals: Sensor=${EXPECTED_SENSOR_INTERVAL}s, Heartbeat=${EXPECTED_HEARTBEAT_INTERVAL}s, Status=${EXPECTED_STATUS_INTERVAL}s"
echo "Tolerance: ±${TOLERANCE}s"
echo "Log File: ${LOG_FILE}"
echo ""

# Check dependencies
if ! command -v mosquitto_sub &> /dev/null; then
    echo "❌ ERROR: mosquitto_sub not found. Please install mosquitto clients."
    exit 1
fi

# Initialize timing tracking
declare -A last_sensor_time
declare -A last_heartbeat_time
declare -A last_status_time
declare -A sensor_intervals
declare -A heartbeat_intervals
declare -A status_intervals

# Initialize counters
declare -A sensor_count
declare -A heartbeat_count
declare -A status_count

architectures=("sync" "async" "full")
for arch in "${architectures[@]}"; do
    last_sensor_time[$arch]=0
    last_heartbeat_time[$arch]=0
    last_status_time[$arch]=0
    sensor_count[$arch]=0
    heartbeat_count[$arch]=0
    status_count[$arch]=0
done

# Create log file
echo "# ESP32-C3 IoT Timing Validation Log - $(date)" > "$LOG_FILE"
echo "# Format: [TIMESTAMP] [UNIX_TIME] TOPIC MESSAGE_TYPE ARCHITECTURE INTERVAL_SINCE_LAST" >> "$LOG_FILE"

echo "⏱️  Starting timing analysis..."
echo "📡 Monitoring esp32c3/+/+/+ for ${VALIDATION_TIME} seconds..."
echo "🕐 This will take $(( VALIDATION_TIME / 60 )) minutes to collect sufficient timing data..."
echo ""

# Function to analyze timing
analyze_timing() {
    local topic="$1"
    local current_time="$2"
    local timestamp="$3"
    
    # Extract architecture from topic
    local arch=""
    if [[ "$topic" == *"/sync" ]]; then
        arch="sync"
    elif [[ "$topic" == *"/async" ]]; then
        arch="async"
    elif [[ "$topic" == *"/full" ]]; then
        arch="full"
    else
        return
    fi
    
    # Determine message type and analyze interval
    if [[ "$topic" == *"/sensor/"* ]]; then
        ((sensor_count[$arch]++))
        if [ "${last_sensor_time[$arch]}" -ne 0 ]; then
            local interval=$((current_time - last_sensor_time[$arch]))
            sensor_intervals[$arch]+="$interval "
            
            # Check if interval is within tolerance
            local diff=$((interval - EXPECTED_SENSOR_INTERVAL))
            local abs_diff=${diff#-}  # absolute value
            
            if [ "$abs_diff" -le "$TOLERANCE" ]; then
                echo "  ✅ SENSOR [$arch] Interval: ${interval}s (expected: ${EXPECTED_SENSOR_INTERVAL}s) ✓"
                echo "[$timestamp] [$current_time] $topic SENSOR $arch $interval GOOD" >> "$LOG_FILE"
            else
                echo "  ⚠️  SENSOR [$arch] Interval: ${interval}s (expected: ${EXPECTED_SENSOR_INTERVAL}s ±${TOLERANCE}s) ⚠️"
                echo "[$timestamp] [$current_time] $topic SENSOR $arch $interval WARNING" >> "$LOG_FILE"
            fi
        else
            echo "  📍 SENSOR [$arch] First message (baseline established)"
            echo "[$timestamp] [$current_time] $topic SENSOR $arch 0 BASELINE" >> "$LOG_FILE"
        fi
        last_sensor_time[$arch]=$current_time
        
    elif [[ "$topic" == *"/heartbeat/"* ]]; then
        ((heartbeat_count[$arch]++))
        if [ "${last_heartbeat_time[$arch]}" -ne 0 ]; then
            local interval=$((current_time - last_heartbeat_time[$arch]))
            heartbeat_intervals[$arch]+="$interval "
            
            local diff=$((interval - EXPECTED_HEARTBEAT_INTERVAL))
            local abs_diff=${diff#-}
            
            if [ "$abs_diff" -le "$TOLERANCE" ]; then
                echo "  ✅ HEARTBEAT [$arch] Interval: ${interval}s (expected: ${EXPECTED_HEARTBEAT_INTERVAL}s) ✓"
                echo "[$timestamp] [$current_time] $topic HEARTBEAT $arch $interval GOOD" >> "$LOG_FILE"
            else
                echo "  ⚠️  HEARTBEAT [$arch] Interval: ${interval}s (expected: ${EXPECTED_HEARTBEAT_INTERVAL}s ±${TOLERANCE}s) ⚠️"
                echo "[$timestamp] [$current_time] $topic HEARTBEAT $arch $interval WARNING" >> "$LOG_FILE"
            fi
        else
            echo "  📍 HEARTBEAT [$arch] First message (baseline established)"
            echo "[$timestamp] [$current_time] $topic HEARTBEAT $arch 0 BASELINE" >> "$LOG_FILE"
        fi
        last_heartbeat_time[$arch]=$current_time
        
    elif [[ "$topic" == *"/status/"* ]]; then
        ((status_count[$arch]++))
        if [ "${last_status_time[$arch]}" -ne 0 ]; then
            local interval=$((current_time - last_status_time[$arch]))
            status_intervals[$arch]+="$interval "
            
            local diff=$((interval - EXPECTED_STATUS_INTERVAL))
            local abs_diff=${diff#-}
            
            if [ "$abs_diff" -le "$TOLERANCE" ]; then
                echo "  ✅ STATUS [$arch] Interval: ${interval}s (expected: ${EXPECTED_STATUS_INTERVAL}s) ✓"
                echo "[$timestamp] [$current_time] $topic STATUS $arch $interval GOOD" >> "$LOG_FILE"
            else
                echo "  ⚠️  STATUS [$arch] Interval: ${interval}s (expected: ${EXPECTED_STATUS_INTERVAL}s ±${TOLERANCE}s) ⚠️"
                echo "[$timestamp] [$current_time] $topic STATUS $arch $interval WARNING" >> "$LOG_FILE"
            fi
        else
            echo "  📍 STATUS [$arch] First message (baseline established)"
            echo "[$timestamp] [$current_time] $topic STATUS $arch 0 BASELINE" >> "$LOG_FILE"
        fi
        last_status_time[$arch]=$current_time
    fi
}

# Function to calculate average and check consistency
calculate_stats() {
    local intervals="$1"
    local expected="$2"
    
    if [ -z "$intervals" ]; then
        echo "N/A"
        return
    fi
    
    local sum=0
    local count=0
    local min=9999
    local max=0
    
    for interval in $intervals; do
        sum=$((sum + interval))
        count=$((count + 1))
        if [ "$interval" -lt "$min" ]; then
            min=$interval
        fi
        if [ "$interval" -gt "$max" ]; then
            max=$interval
        fi
    done
    
    if [ "$count" -gt 0 ]; then
        local avg=$((sum / count))
        local diff=$((avg - expected))
        local abs_diff=${diff#-}
        
        if [ "$abs_diff" -le "$TOLERANCE" ]; then
            echo "avg=${avg}s min=${min}s max=${max}s count=${count} ✅"
        else
            echo "avg=${avg}s min=${min}s max=${max}s count=${count} ⚠️"
        fi
    else
        echo "N/A"
    fi
}

# Start monitoring
timeout "$VALIDATION_TIME" mosquitto_sub -h "$BROKER_IP" -t "esp32c3/+/+/+" -v 2>/dev/null | while IFS= read -r line; do
    current_time=$(date +%s)
    timestamp=$(date '+%H:%M:%S')
    
    # Parse topic and payload
    topic=$(echo "$line" | cut -d' ' -f1)
    payload=$(echo "$line" | cut -d' ' -f2-)
    
    echo "[$timestamp] $topic"
    
    # Analyze timing for this message
    analyze_timing "$topic" "$current_time" "$timestamp"
    
    echo ""
done

echo ""
echo "=== Timing Analysis Summary ==="
echo ""

# Detailed statistics for each architecture
for arch in "${architectures[@]}"; do
    echo "🏗️  Architecture: $arch (main-$([ "$arch" = "sync" ] && echo "nodeps" || ([ "$arch" = "async" ] && echo "min" || echo "app")))"
    echo "   📊 Message Counts:"
    echo "      Sensor:    ${sensor_count[$arch]}"
    echo "      Heartbeat: ${heartbeat_count[$arch]}"
    echo "      Status:    ${status_count[$arch]}"
    echo "   ⏱️  Timing Statistics:"
    echo "      Sensor (${EXPECTED_SENSOR_INTERVAL}s):    $(calculate_stats "${sensor_intervals[$arch]:-}" $EXPECTED_SENSOR_INTERVAL)"
    echo "      Heartbeat (${EXPECTED_HEARTBEAT_INTERVAL}s): $(calculate_stats "${heartbeat_intervals[$arch]:-}" $EXPECTED_HEARTBEAT_INTERVAL)"
    echo "      Status (${EXPECTED_STATUS_INTERVAL}s):    $(calculate_stats "${status_intervals[$arch]:-}" $EXPECTED_STATUS_INTERVAL)"
    echo ""
done

# Overall assessment
echo "🎯 Timing Standardization Assessment:"

# Check if we have enough data
total_sensor_messages=$((sensor_count[sync] + sensor_count[async] + sensor_count[full]))
total_heartbeat_messages=$((heartbeat_count[sync] + heartbeat_count[async] + heartbeat_count[full]))
total_status_messages=$((status_count[sync] + status_count[async] + status_count[full]))

if [ "$total_sensor_messages" -lt 3 ] || [ "$total_heartbeat_messages" -lt 2 ] || [ "$total_status_messages" -lt 1 ]; then
    echo "⚠️  WARNING: Insufficient data for reliable timing analysis"
    echo "   📊 Collected: ${total_sensor_messages} sensor, ${total_heartbeat_messages} heartbeat, ${total_status_messages} status messages"
    echo "   📋 Recommendation: Run validation for longer duration or check device connectivity"
else
    # Count warnings in log file
    warning_count=$(grep -c "WARNING" "$LOG_FILE" 2>/dev/null || echo "0")
    good_count=$(grep -c "GOOD" "$LOG_FILE" 2>/dev/null || echo "0")
    total_interval_checks=$((warning_count + good_count))
    
    if [ "$total_interval_checks" -gt 0 ]; then
        success_rate=$(( (good_count * 100) / total_interval_checks ))
        echo "   ✅ Timing Compliance: ${good_count}/${total_interval_checks} intervals within tolerance (${success_rate}%)"
        
        if [ "$success_rate" -ge 90 ]; then
            echo "🎉 SUCCESS: Timing standardization is working excellently!"
        elif [ "$success_rate" -ge 75 ]; then
            echo "✅ GOOD: Timing standardization is working well with minor variations"
        else
            echo "❌ FAILURE: Timing standardization needs improvement"
        fi
    else
        echo "⚠️  Unable to calculate timing compliance (no interval data)"
    fi
fi

echo ""
echo "📋 Detailed timing log: $LOG_FILE"
echo "🔄 To run message format validation: ./validate_messages.sh"

# Recommendations
echo ""
echo "💡 Recommendations:"
if [ "$total_sensor_messages" -eq 0 ]; then
    echo "   🔧 No sensor messages received - check sensor task implementation"
fi
if [ "$total_heartbeat_messages" -eq 0 ]; then
    echo "   🔧 No heartbeat messages received - check heartbeat task implementation"
fi
if [ "$total_status_messages" -eq 0 ]; then
    echo "   🔧 No status messages received - check status reporting implementation"
fi

if [ "$warning_count" -gt 0 ]; then
    echo "   🔧 Timing warnings detected - review IMPLEMENTATION_CHANGES_GUIDE.md for timing standardization"
    echo "   🔧 Check that all applications use StandardTimingConfig with 30s/60s/120s intervals"
fi

echo ""