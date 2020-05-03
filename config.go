package main

import (
	"fmt"
	"github.com/spf13/viper"
	"log"
	"time"
)

type Configuration struct {
	MQTTHostname            string        `mapstructure:"mqtt_hostname"`
	MQTTPort                int           `mapstructure:"mqtt_port"`
	MQTTUsername            string        `mapstructure:"mqtt_username"`
	MQTTPassword            string        `mapstructure:"mqtt_password"`
	MQTTClientId            string        `mapstructure:"mqtt_client_id"`
	MeasureInterval         time.Duration `mapstructure:"measure_interval"`
	BLEMeasureInterval      time.Duration `mapstructure:"ble_measure_interval"`
	BME280I2CBusId          int           `mapstructure:"bme280_i2c_bus_id"`
	BME280I2CDeviceAddress  uint8         `mapstructure:"bme280_i2c_device_address"`
	BLEDeviceAddr           string        `mapstructure:"ble_device_address"`
	BLEDeviceCharacteristic string        `mapstructure:"ble_characteristic_uuid"`
}

func setupConfig(hostname string) *Configuration {
	viper.SetConfigName("sensor_mqtt")
	viper.SetConfigType("yaml")
	//viper.AddConfigPath(".")
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
