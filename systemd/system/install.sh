#!/usr/bin/env bash
set -e

name="mqtt2influxdb"

dir=$(basename "$(pwd)")
if [ "$dir" == "systemd" ] || [ "$dir" == "system" ]; then
    echo "run from main directiory like this: ./systemd/system/install.sh"
    exit 1
fi

nice cargo build --release --locked

# systemd
sudo mkdir -p /usr/local/lib/systemd/system/
sudo cp -uv "systemd/system/systemd.service" "/usr/local/lib/systemd/system/$name.service"
sudo systemctl daemon-reload

# stop, replace and start new version
sudo systemctl stop "$name.service"
sudo cp -v "target/release/$name" /usr/local/bin

sudo systemctl enable --now "$name.service"
