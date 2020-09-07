#!/usr/bin/env sh

userdel sensor_mqtt || echo "Unable delete user 'sensor_mqtt'"
groupdel sensor_mqtt || echo "Unable delete group 'sensor_mqtt'"
