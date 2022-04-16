#!/usr/bin/env sh
useradd -s /bin/false -M -G i2c sensor_mqtt || echo "User already exists"
usermod -g i2c sensor_mqtt
