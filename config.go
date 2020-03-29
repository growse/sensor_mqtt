package main

import (
	"fmt"
	"github.com/spf13/viper"
	"log"
	"time"
)

func setupConfig(hostname string) *Configuration {
	viper.SetConfigName("sensor_mqtt")
	viper.SetConfigType("yaml")
	viper.AddConfigPath(".")
	viper.AddConfigPath("/etc/sensor_mqtt/")

	err := viper.ReadInConfig()
	// Find and read the Configuration file
	if err != nil { // Handle errors reading the Configuration file
		log.Fatalf("Fatal error Configuration file: %v", err)
	}
	defaultConfig := &Configuration{
		MQTTHostname:            "mqtt",
		MQTTPort:                1883,
		MQTTUsername:            "sensor_mqtt",
		MQTTPassword:            "password",
		MQTTClientId:            fmt.Sprintf("%v_sensor_mqtt", hostname),
		MeasureInterval:         30 * time.Second,
		BLEMeasureInterval:      10 * time.Minute,
		BME280I2CDeviceAddress:  0x76,
		BME280I2CBusId:          1,
		BLEDeviceAddr:           "",
		BLEDeviceCharacteristic: "b42e4dcc-ade7-11e4-89d3-123b93f75cba",
	}
	err = viper.Unmarshal(&defaultConfig)
	if err != nil {
		log.Fatalf("Fatal error Configuration file: %v", err)
	}
	return defaultConfig
}
