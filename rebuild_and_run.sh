#!/bin/bash
# Rebuild and relaunch BitVault Desktop app

cd "$(dirname "$0")"

echo "Building BitVault Desktop..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful! Killing existing instances..."
    pkill -f bitvault-app 2>/dev/null
    sleep 1
    echo "Launching BitVault..."
    ./target/release/bitvault-app > /tmp/bitvault_app.log 2>&1 &
    echo "App launched! (PID: $!)"
else
    echo "Build failed! Not launching."
    exit 1
fi
