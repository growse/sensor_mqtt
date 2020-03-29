package main

import (
	"fmt"
	"github.com/d2r2/go-bsbmp"
	"github.com/d2r2/go-logger"
	mqtt "github.com/eclipse/paho.mqtt.golang"
	"log"
	"time"
)
import "github.com/d2r2/go-i2c"

type BME280Measurements struct {
	Temperature float32
	Pressure    float32
	Humidity    float32
}

const SensorAccuracy = bsbmp.ACCURACY_HIGH

var humiditySupported = true

func getI2CConnection(deviceAddress uint8, busId int) *i2c.I2C {
	_ = logger.ChangePackageLogLevel("i2c", logger.InfoLevel)
	i2cConnection, err := i2c.NewI2C(deviceAddress, busId)
	if err != nil {
		log.Fatal(err)
	}
	return i2cConnection

}

func GetBME280Sensor(deviceAddress uint8, busId int) *bsbmp.BMP {
	i2c := getI2CConnection(deviceAddress, busId)
	_ = logger.ChangePackageLogLevel("bsbmp", logger.InfoLevel)
	sensor, err := bsbmp.NewBMP(bsbmp.BME280, i2c)
	if err != nil {
		log.Fatalf("Unable to connect to BME280 Sensor: %v", err)
	}
	return sensor
}

func collectMeasurements(sensor *bsbmp.BMP) BME280Measurements {
	temperature, err := sensor.ReadTemperatureC(SensorAccuracy)
	if err != nil {
		log.Fatal(err)
	}
	pressure, err := sensor.ReadPressurePa(SensorAccuracy)
	if err != nil {
		log.Fatal(err)
	}
	var humidity float32
	if humiditySupported {
		supported, measuredHumidity, err := sensor.ReadHumidityRH(SensorAccuracy)
		if !supported {
			log.Print("Humidity not supported. Skipping from now on and reporting '0'")
			humiditySupported = false
		}
		if err != nil {
			log.Fatal(err)
		}
		humidity = measuredHumidity
	}
	return BME280Measurements{Temperature: temperature, Pressure: pressure / 100, Humidity: humidity}
}

func startBME280MeasurementCollectorLoop(interval time.Duration, deviceAddress uint8, busId int, quit chan bool) chan BME280Measurements {
	bme280Sensor := GetBME280Sensor(deviceAddress, busId)
	ticker := time.NewTicker(interval)
	dataChannel := make(chan BME280Measurements)
	go bme280Loop(dataChannel, bme280Sensor, ticker, quit)
	return dataChannel
}

func bme280Loop(measurements chan<- BME280Measurements, bme280Sensor *bsbmp.BMP, ticker *time.Ticker, quit chan bool) {
	defer ticker.Stop()
	log.Println("BME280 Collector loop")
	measurements <- collectMeasurements(bme280Sensor)
L:
	for {
		select {
		case <-ticker.C:
			measurements <- collectMeasurements(bme280Sensor)
			break
		case <-quit:
			break L
		}
	}
	log.Println("Exiting BME280 Collector loop")
}

func startBME280MQTTPublisherLoop(mqttClient mqtt.Client, dataChan chan BME280Measurements, topicPrefix string, quit chan bool) {
	log.Println("BME280 Publisher loop")
L:
	for {
		select {
		case data := <-dataChan:
			var token mqtt.Token
			token = mqttClient.Publish(topicPrefix+"/temperature", 0, false, fmt.Sprintf("%.2f", data.Temperature))
			token.Wait()
			if token.Error() != nil {
				log.Printf("Error publishing metric to MQTT: %v", token.Error())
			}
			token = mqttClient.Publish(topicPrefix+"/pressure", 0, false, fmt.Sprintf("%.2f", data.Pressure))
			token.Wait()
			if token.Error() != nil {
				log.Printf("Error publishing metric to MQTT: %v", token.Error())
			}
			token = mqttClient.Publish(topicPrefix+"/humidity", 0, false, fmt.Sprintf("%.2f", data.Humidity))
			token.Wait()
			if token.Error() != nil {
				log.Printf("Error publishing metric to MQTT: %v", token.Error())
			}
			break
		case <-quit:
			log.Println("Request to exit BME280 Publisher loop")
			break L
		}
	}
	log.Println("Exiting BME280 Publisher loop")
}
