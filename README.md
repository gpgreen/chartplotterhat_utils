# chartplotterhat_utils
Utilities to work with the ![Chart Plotter
Hat](https://github.com/gpgreen/chart_plotter_hat) and the ![Chart
Plotter Hat Firmware](https://github.com/gpgreen/power-monitor)

Intended for use with Raspberry PI boat computer project.

## Raspberry Pi tools

### shutdown_monitor
The binary `shutdown_monitor` is used to control and monitor the GPIO
pins on the Raspberry Pi to work in concert with the firmware. This
binary is copied to /usr/bin. The systemd service file
`shutdown_monitor.service` and is copied to /lib/systemd/service.

The process requires 2 environment variables to be set, which are in the systemd service file.

Enabling and starting the service using systemd
```
systemctl enable shutdown_monitor.service
systemctl start shutdown_monitor.serviceg
```

To disable the service from starting at boot
```
systemctl disable shutdown_monitor.service
```

### spitool
The binary `spitool` works with the spi functionality on the
hardware. There are several functions that can be accessed.

* firmware version - will be printed on stdout whenever the binary runs

* eeprom write toggle - using the argument `-e` or `--eeprom` will
  toggle the write access of the onboard eeprom

* adc channels - to be implemented

## Raspberry Pi Device Tree Overlay
The [device tree overlay](eeprom/chart-plotter-hat-overlay.dts) is
used to configure the hardware while booting the Pi.  The binary
overlay used can be generated via:
```
dtc -@ -I dts -O dtb -o chart-plotter-hat.dtbo chart-plotter-hat-overlay.dts
```
Copy this device tree binary overlay to /boot/overlays/ on the Raspberry Pi

## EEPROM on board the hat
The eeprom on the hat can be loaded with required data so that the
pins on the hat are configured, the device tree overlay is loaded, and
the CAN driver probed and loaded. The eeprom data is prepared and
loaded with the use of utilities in the [Raspberry
Hat](https://github.com/raspberrypi/hats/tree/master/eepromutils). Use
the `eeprom/settings.txt` file and include the overlay name when
creating the eeprom data file.
```
eepmake eeprom_settings.txt eeprom.eep chart-plotter-hat
```

License
-------

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
