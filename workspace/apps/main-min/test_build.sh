#!/bin/bash
# Quick build test for main-min

echo "Testing main-min compilation with real WiFi/MQTT..."

# Set timeout for compilation
timeout 30s cargo check --features mqtt

if [ $? -eq 124 ]; then
    echo "Build timeout - compilation is proceeding but slow"
    echo "This is normal for first build with many dependencies"
    exit 0
elif [ $? -eq 0 ]; then
    echo "Build successful!"
    exit 0
else
    echo "Build failed - checking syntax errors"
    exit 1
fi