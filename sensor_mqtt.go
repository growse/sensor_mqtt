package main

import (
	"fmt"
	"log"
	"os"
	"time"
)

type Configuration struct {
	MQTTHostname           string        `mapstructure:"mqtt_hostname"`
	MQTTPort               int           `mapstructure:"mqtt_port"`
	MQTTUsername           string        `mapstructure:"mqtt_username"`
	MQTTPassword           string        `mapstructure:"mqtt_password"`
	MQTTClientId           string        `mapstructure:"mqtt_client_id"`
	MeasureInterval        time.Duration `mapstructure:"measure_interval"`
	BME280I2CBusId         int           `mapstructure:"bme280_i2c_bus_id"`
	BME280I2CDeviceAddress uint8         `mapstructure:"bme280_i2c_device_address"`
}

func main() {
	hostname, err := os.Hostname()
	if err != nil {
		log.Fatalf("Error retrieving hostname: %v", err)
	}
	log.Printf("Reading config")
	config := setupConfig(hostname)

	i2cConnection := getI2CConnection(config)
	defer i2cConnection.Close()
	bme280Sensor := getBME280Sensor(i2cConnection)

	mqttClient := getMQTTClient(config)

	if token := mqttClient.Connect(); token.Wait() && token.Error() != nil {
		log.Fatalf("Error connecting to MQTT: %v", token.Error())
	}
	defer mqttClient.Disconnect(0)

	dataChannel := startMeasurementCollectorLoop(config.MeasureInterval, bme280Sensor)

	startMQTTPublisherLoop(mqttClient, dataChannel, fmt.Sprintf("sensors/%s", hostname))
}
