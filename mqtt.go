package main

import (
	"fmt"
	mqtt "github.com/eclipse/paho.mqtt.golang"
	"log"
	"time"
)

func getMQTTClient(config *Configuration) mqtt.Client {
	broker := fmt.Sprintf("tcp://%s:%d", config.MQTTHostname, config.MQTTPort)
	log.Printf("MQTT broker: %v", broker)
	mqttClientOptions := mqtt.
		NewClientOptions().
		AddBroker(broker).
		SetUsername(config.MQTTUsername).
		SetPassword(config.MQTTPassword).
		SetProtocolVersion(4).
		SetClientID(config.MQTTClientId).
		SetAutoReconnect(true).
		SetKeepAlive(2 * time.Second).
		SetPingTimeout(1 * time.Second).
		SetConnectTimeout(1 * time.Second).SetConnectionLostHandler(connectionLostHandler).SetOnConnectHandler(onConnectHandler)
	client := mqtt.NewClient(mqttClientOptions)
	return client
}

func onConnectHandler(client mqtt.Client) {
	log.Printf("MQTT connected")
}

func connectionLostHandler(client mqtt.Client, e error) {
	log.Printf("MQTT connection lost: %v", e)
}

func startMQTTPublisherLoop(mqttClient mqtt.Client, dataChan chan BME280Measurements, topicPrefix string) {
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
		}
	}
}
