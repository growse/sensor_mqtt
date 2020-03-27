package main

import (
	"github.com/d2r2/go-bsbmp"
	"github.com/d2r2/go-logger"
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

func getI2CConnection(configuration *Configuration) *i2c.I2C {
	_ = logger.ChangePackageLogLevel("i2c", logger.InfoLevel)
	i2cConnection, err := i2c.NewI2C(configuration.BME280I2CDeviceAddress, configuration.BME280I2CBusId)
	if err != nil {
		log.Fatal(err)
	}
	return i2cConnection

}

func getBME280Sensor(i2cConnection *i2c.I2C) *bsbmp.BMP {
	_ = logger.ChangePackageLogLevel("bsbmp", logger.InfoLevel)
	sensor, err := bsbmp.NewBMP(bsbmp.BME280, i2cConnection)
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

func startMeasurementCollectorLoop(interval time.Duration, bmp *bsbmp.BMP) chan BME280Measurements {
	ticker := time.NewTicker(interval)
	dataChannel := make(chan BME280Measurements)
	go loop(dataChannel, bmp, ticker)
	return dataChannel
}

func loop(measurements chan<- BME280Measurements, bmp *bsbmp.BMP, ticker *time.Ticker) {
	defer ticker.Stop()
	for {
		select {
		case <-ticker.C:
			measurements <- collectMeasurements(bmp)
		}
	}
}
