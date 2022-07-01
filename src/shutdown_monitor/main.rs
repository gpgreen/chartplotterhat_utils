use chartplotterhat_utils::ChartPlotterHatPower;
use hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    //    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin,
};
use linux_embedded_hal as hal;
use std::{env, num::ParseIntError, process::Command, thread, time};

// Environmental variables required by this executable
const PKILL_DELAY_KEY: &str = "OPENCPN_PKILL_DELAY";
const PKILL_USER_KEY: &str = "OPENCPN_USER";

// raspberry pi pin numbers used by this executable
const SHUTDOWN_PIN_NUM: u32 = 22;
const MCU_RUNNING_PIN_NUM: u32 = 23;

fn parse_delay(num_str: &str) -> Result<u64, Error> {
    match num_str.parse::<u64>() {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::Parse(e)),
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse(ParseIntError),
    EnvVarNotSet(&'static str),
}

fn main() -> Result<(), Error> {
    // get environment variables
    let mut pkill_delay: Option<u64> = None;
    let mut pkill_user: Option<String> = None;

    for (key, value) in env::vars() {
        if key == PKILL_DELAY_KEY {
            pkill_delay = Some(parse_delay(&value)?);
        } else if key == PKILL_USER_KEY {
            pkill_user = Some(value);
        }
    }
    if pkill_delay.is_none() {
        return Err(Error::EnvVarNotSet(PKILL_DELAY_KEY));
    }
    if pkill_user.is_none() {
        return Err(Error::EnvVarNotSet(PKILL_USER_KEY));
    }

    // get the gpio chip
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();

    // configure Digital I/O pin to be used as mcu_running pin
    let line = chip.get_line(MCU_RUNNING_PIN_NUM).unwrap();
    let handle = line
        .request(LineRequestFlags::OUTPUT, 1, "shutdown_monitor")
        .unwrap();
    let mcu_running = CdevPin::new(handle).unwrap();

    // configure Digital I/O pin to be used as shutdown pin
    let line = chip.get_line(SHUTDOWN_PIN_NUM).unwrap();
    let handle = line
        .request(LineRequestFlags::INPUT, 0, "shutdown_monitor")
        .unwrap();
    let shutdown = CdevPin::new(handle).unwrap();

    // create the ChartPlotterHatPower device
    let mut cph = ChartPlotterHatPower::new(shutdown, mcu_running);

    // set the pin to show we are running
    cph.set_running().unwrap();

    // now watch for shutdown signal
    loop {
        if cph.shutdown_signal().unwrap() {
            // run the pkill
            println!("Detected shutdown signal, powering off..");
            Command::new("/usr/bin/pkill")
                .arg("-u")
                .arg(pkill_user.expect("pkill_user was invalid"))
                .arg("opencpn")
                .output()
                .expect("failed to execute pkill");
            // sleep after the pkill to let opencpn quit gracefully
            thread::sleep(time::Duration::from_millis(
                pkill_delay.expect("pkill_delay was invalid") * 1000,
            ));
            // run the shutdown
            Command::new("/usr/bin/sudo")
                .arg("/sbin/poweroff")
                .output()
                .expect("failed to execute poweroff");
            return Ok(());
        }
        thread::sleep(time::Duration::from_millis(1000));
    }
}
