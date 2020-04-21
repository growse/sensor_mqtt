#!/usr/bin/env sh
setcap cap_net_raw,cap_net_admin+ep /usr/bin/sensor_mqtt || echo "Unable to setcap on /usr/bin/sensor_mqtt"
