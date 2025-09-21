#!/usr/bin/env python3
"""
Quick test script to identify working vs broken modules
"""

import subprocess
import sys
from pathlib import Path

def test_module_build(module):
    """Test if a module builds successfully"""
    try:
        result = subprocess.run(
            ["cargo", "build", "--release", "-p", module],
            capture_output=True,
            text=True,
            timeout=120
        )
        return result.returncode == 0, result.stderr
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, str(e)

def test_binary_build(binary):
    """Test if a binary builds successfully"""
    try:
        result = subprocess.run(
            ["cargo", "build", "--release", "--bin", binary],
            capture_output=True,
            text=True,
            timeout=120
        )
        return result.returncode == 0, result.stderr
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, str(e)

def main():
    print("🔍 Quick Module Build Test")
    print("=" * 50)
    
    # Test modules
    modules = [
        "blinky",
        "bme280-embassy", 
        "wifi-embassy",
        "wifi-synchronous",
        "serial-console-embassy",
        "mqtt-embassy",
        "main-app",
        "iot-common",
        "iot-container",
        "iot-hal",
        "iot-performance"
    ]
    
    working = []
    broken = []
    
    for module in modules:
        print(f"Testing {module}...", end=" ")
        success, error = test_module_build(module)
        if success:
            print("✅ WORKS")
            working.append(module)
        else:
            print("❌ BROKEN")
            broken.append((module, error[:100] if error else "Unknown error"))
    
    # Test specific binaries
    print(f"\n🎯 Testing Binary Targets")
    print("-" * 30)
    
    binaries = ["main", "main_container"]
    for binary in binaries:
        print(f"Testing {binary}...", end=" ")
        success, error = test_binary_build(binary)
        if success:
            print("✅ WORKS")
            working.append(f"{binary} (binary)")
        else:
            print("❌ BROKEN")
            broken.append((f"{binary} (binary)", error[:100] if error else "Unknown error"))
    
    print(f"\n📊 RESULTS")
    print("=" * 50)
    print(f"✅ Working modules ({len(working)}):")
    for item in working:
        print(f"  • {item}")
    
    print(f"\n❌ Broken modules ({len(broken)}):")
    for item, error in broken:
        print(f"  • {item}")
        print(f"    Error: {error}")
    
    print(f"\n🎯 DEPLOYMENT RECOMMENDATION:")
    if "main" in [w.replace(" (binary)", "") for w in working]:
        print("✅ Deploy with: cargo build --release --bin main")
        print("✅ Original working architecture available")
    else:
        print("❌ No working deployment target found")
    
    return len(broken) == 0

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)