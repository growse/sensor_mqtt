DEBNAME := sensor_mqtt
VERSION := v0.0.2
APPDESCRIPTION := Daemon to publish sensor metrics to MQTT
APPURL := https://github.com/growse/sensor_mqtt
ARCH := amd64 arm

# Setup
BUILD_NUMBER ?= 0
DEBVERSION := $(VERSION:v%=%)-$(BUILD_NUMBER)

# Let's map from go architectures to deb architectures, because they're not the same!
DEB_arm_ARCH := armhf
DEB_amd64_ARCH := amd64

# CC Toolchain mapping
CC_FOR_LINUX_ARM := arm-linux-gnueabi-gcc

.EXPORT_ALL_VARIABLES:

.PHONY: package
package: $(addsuffix .deb, $(addprefix $(DEBNAME)_$(DEBVERSION)_, $(foreach a, $(ARCH), $(a))))

.PHONY: build
build: $(addprefix dist/$(DEBNAME)_linux_, $(foreach a, $(ARCH), $(a)))

dist/$(DEBNAME)_linux_%: $(wildcard *.go)
	GOOS=linux GOARCH=$* go build -o dist/$(DEBNAME)_linux_$*

$(DEBNAME)_$(DEBVERSION)_%.deb: dist/$(DEBNAME)_linux_%
	chmod +x $<
	bundle exec fpm -f -s dir -t deb -n $(DEBNAME) --description "$(APPDESCRIPTION)" --url $(APPURL) --deb-changelog CHANGELOG.md --prefix / -a $(DEB_$*_ARCH) -v $(DEBVERSION) --before-install deb_scripts/before_install.sh --before-upgrade deb_scripts/before_upgrade.sh --after-remove deb_scripts/after_remove.sh --after-install deb_scripts/after_install.sh --after-upgrade deb_scripts/after_upgrade.sh --deb-systemd sensor_mqtt.service --config-files /etc/sensor_mqtt/sensor_mqtt.yaml sensor_mqtt.yaml=/etc/sensor_mqtt/sensor_mqtt.yaml $<=/usr/bin/sensor_mqtt

.PHONY: clean
clean:
	rm -f *.deb
	rm -rf dist
