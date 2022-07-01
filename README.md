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
systemctl start shutdown_monitor.service
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
