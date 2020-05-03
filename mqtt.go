package main

import (
	"fmt"
	mqtt "github.com/eclipse/paho.mqtt.golang"
	"log"
	"time"
)

func getMQTTClient(config *Configuration, statusTopic string) mqtt.Client {
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
		SetKeepAlive(2*time.Second).
		SetPingTimeout(1*time.Second).
		SetConnectTimeout(1*time.Second).
		SetConnectionLostHandler(connectionLostHandler).
		SetOnConnectHandler(onConnectHandler).
		SetWill(statusTopic, "offline", 1, true)

	client := mqtt.NewClient(mqttClientOptions)
	return client
}

func onConnectHandler(client mqtt.Client) {
	log.Printf("MQTT connected")
	optionsReader := client.OptionsReader()
	willTopic := optionsReader.WillTopic()
	client.Publish(willTopic, 1, true, "online")
}

func connectionLostHandler(client mqtt.Client, e error) {
	log.Printf("MQTT connection lost: %v", e)
	time.Sleep(5 * time.Second)
}
