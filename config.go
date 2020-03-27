package main

import (
	"fmt"
	"github.com/spf13/viper"
	"log"
	"time"
)

func setupConfig(hostname string) *Configuration {
	viper.SetConfigName("sensor_mqtt")
	viper.AddConfigPath("/etc/sensor_mqtt/")
	viper.AddConfigPath(".")
	err := viper.ReadInConfig()
	// Find and read the Configuration file
	if err != nil { // Handle errors reading the Configuration file
		log.Fatalf("Fatal error Configuration file: %v", err)
	}
	defaultConfig := &Configuration{
		MQTTHostname:           "mqtt",
		MQTTPort:               1883,
		MQTTUsername:           "sensor_mqtt",
		MQTTPassword:           "password",
		MQTTClientId:           fmt.Sprintf("%v_sensor_mqtt", hostname),
		MeasureInterval:        30 * time.Second,
		BME280I2CDeviceAddress: 0x76,
		BME280I2CBusId:         1,
	}
	err = viper.Unmarshal(&defaultConfig)
	if err != nil {
		log.Fatalf("Fatal error Configuration file: %v", err)
	}
	return defaultConfig
}
