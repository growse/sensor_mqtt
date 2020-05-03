package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
)



func main() {
	sigs := make(chan os.Signal, 1)
	quitFromSignalHandler := make(chan bool, 1)
	routineQuit := make(chan bool, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	go signalHandler(sigs, quitFromSignalHandler)
	hostname, err := os.Hostname()
	if err != nil {
		log.Fatalf("Error retrieving hostname: %v", err)
	}
	log.Printf("Reading config")
	config := setupConfig(hostname)

	topic := fmt.Sprintf("sensors/%s", hostname)
	statusTopic := fmt.Sprintf("%s/status", topic)
	mqttClient := getMQTTClient(config, statusTopic)

	if token := mqttClient.Connect(); token.Wait() && token.Error() != nil {
		log.Fatalf("Error connecting to MQTT: %v", token.Error())
	}

	defer mqttClient.Disconnect(0)

	bme280DataChannel := startBME280MeasurementCollectorLoop(config.MeasureInterval, config.BME280I2CDeviceAddress, config.BME280I2CBusId, routineQuit)
	go startBME280MQTTPublisherLoop(mqttClient, bme280DataChannel, topic, quitFromSignalHandler)

	if config.BLEDeviceAddr != "" {
		bleDataChannel := startBLEMeasurementCollectorLoop(config.BLEMeasureInterval, config.BLEDeviceAddr, config.BLEDeviceCharacteristic, routineQuit)
		go startBLEMQTTPublisherLoop(mqttClient, bleDataChannel, topic, quitFromSignalHandler)
	}
	<-quitFromSignalHandler
	routineQuit <- true
	close(routineQuit)
}

func signalHandler(sigs chan os.Signal, quit chan bool) {
	sig := <-sigs
	log.Printf("Received signal: %v\n", sig)
	quit <- true
	close(quit)
}
