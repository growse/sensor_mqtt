package main

import (
	"context"
	"encoding/binary"
	"fmt"
	mqtt "github.com/eclipse/paho.mqtt.golang"
	"github.com/go-ble/ble"
	"github.com/go-ble/ble/examples/lib/dev"
	"log"
	"strings"
	"time"
)

type BLEMeasurements struct {
	Temperature    float32
	Pressure       float32
	Humidity       float32
	RadonShortTerm float32
	RadonLongTerm  float32
}

func readBLEMeausurements(btleMacAddress string, characteristicUUID string) (*BLEMeasurements, error) {
	uuid, err := ble.Parse(characteristicUUID)
	if err != nil {
		log.Fatalf("Invalid characteristic UUID %v", characteristicUUID)
	}
	d, err := dev.NewDevice("default")
	if err != nil {
		return nil, fmt.Errorf("can't get new device : %s", err)
	}
	ble.SetDefaultDevice(d)
	filter := func(a ble.Advertisement) bool {
		return strings.ToUpper(a.Addr().String()) == strings.ToUpper(btleMacAddress)
	}
	duration := 30 * time.Second
	ctx := ble.WithSigHandler(context.WithTimeout(context.Background(), duration))
	cln, err := ble.Connect(ctx, filter)
	if err != nil {
		return nil, fmt.Errorf("can't connect : %s", err)
	}
	// Make sure we had the chance to print out the message.
	done := make(chan struct{})
	// Normally, the connection is disconnected by us after our exploration.
	// However, it can be asynchronously disconnected by the remote peripheral.
	// So we wait(detect) the disconnection in the go routine.
	go func() {
		<-cln.Disconnected()
		fmt.Printf("BLE Client [%s] is disconnected \n", cln.Addr())
		close(done)
	}()

	p, err := cln.DiscoverProfile(true)
	charToFind := ble.Characteristic{UUID: uuid}
	char := p.FindCharacteristic(&charToFind)
	if char == nil {
		return nil, fmt.Errorf("Error finding characteristic: %v", uuid)
	}
	charVals, err := cln.ReadCharacteristic(char)
	if err != nil {
		return nil, fmt.Errorf("Error reading characteristic: %v", err)
	}
	measurements := BLEMeasurements{
		Temperature:    float32(binary.LittleEndian.Uint16(charVals[8:10])) / 100.0,
		Pressure:       float32(binary.LittleEndian.Uint16(charVals[10:12])) / 50,
		Humidity:       float32(charVals[1] / 2.0),
		RadonShortTerm: float32(binary.LittleEndian.Uint16(charVals[4:6])),
		RadonLongTerm:  float32(binary.LittleEndian.Uint16(charVals[6:8])),
	}

	_ = cln.CancelConnection()
	_ = d.Stop()
	return &measurements, nil
}

func startBLEMeasurementCollectorLoop(interval time.Duration, btleMacAddress string, characteristicUUID string, quit chan bool) chan BLEMeasurements {
	ticker := time.NewTicker(interval)
	dataChannel := make(chan BLEMeasurements)
	go bleLoop(dataChannel, ticker, btleMacAddress, characteristicUUID, quit)
	return dataChannel
}

func bleLoop(measurements chan<- BLEMeasurements, ticker *time.Ticker, btleMacAddress string, characteristicUUID string, quit chan bool) {
	log.Println("BLE Collector loop")
	defer ticker.Stop()
	m, _ := readBLEMeausurements(btleMacAddress, characteristicUUID)
	if m != nil {
		measurements <- *m
	}
L:
	for {
		select {
		case <-ticker.C:
			m, _ := readBLEMeausurements(btleMacAddress, characteristicUUID)
			if m != nil {
				measurements <- *m
			}
			break
		case <-quit:
			break L
		}
	}
	log.Println("Exiting BLE Collector loop")
}

func startBLEMQTTPublisherLoop(mqttClient mqtt.Client, dataChan chan BLEMeasurements, topicPrefix string, quit chan bool) {
	log.Println("BLE Publisher loop")
L:
	for {
		select {
		case data := <-dataChan:
			var token mqtt.Token
			token = mqttClient.Publish(topicPrefix+"/radon_short", 0, false, fmt.Sprintf("%.2f", data.RadonShortTerm))
			token.Wait()
			if token.Error() != nil {
				log.Printf("Error publishing metric to MQTT: %v", token.Error())
			}
			token = mqttClient.Publish(topicPrefix+"/radon_long", 0, false, fmt.Sprintf("%.2f", data.RadonLongTerm))
			token.Wait()
			if token.Error() != nil {
				log.Printf("Error publishing metric to MQTT: %v", token.Error())
			}
			break
		case <-quit:
			break L
		}
	}
}
