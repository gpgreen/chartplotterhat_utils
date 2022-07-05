use anyhow::{Context, Result};
use chartplotterhat_utils::ChartPlotterHatPower;
use hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    CdevPin, Delay,
};
use linux_embedded_hal as hal;
use std::{env, process::Command, thread, time};
use thiserror::Error;

// constants for devices
const GPIO_PIN_OWNER: &str = "shutdown_monitor";
const GPIOCHIP: &str = "/dev/gpiochip0";

// Environmental variables required by this executable
const PKILL_DELAY_KEY: &str = "OPENCPN_PKILL_DELAY";
const PKILL_USER_KEY: &str = "OPENCPN_USER";

// raspberry pi pin numbers used by this executable
const SHUTDOWN_PIN_NUM: u32 = 22;
const MCU_RUNNING_PIN_NUM: u32 = 23;

#[derive(Error, Debug)]
pub enum Error {
    #[error("transparent")]
    Io(#[from] std::io::Error),
    #[error("transparent")]
    Parse(#[from] std::num::ParseIntError),
    #[error("environment variable {0} not found")]
    EnvVarNotSet(&'static str),
}

fn main() -> Result<()> {
    // get environment variables
    let mut pkill_delay: Option<u64> = None;
    let mut pkill_user: Option<String> = None;
    for (key, value) in env::vars() {
        if key == PKILL_DELAY_KEY {
            pkill_delay = Some(
                value
                    .parse::<u64>()
                    .with_context(|| format!("{} requires delay in seconds", PKILL_DELAY_KEY))?,
            );
        } else if key == PKILL_USER_KEY {
            pkill_user = Some(value);
        }
    }
    // check that we got our required environment vars
    let pkill_delay = pkill_delay.ok_or(Error::EnvVarNotSet(PKILL_DELAY_KEY))?;
    let pkill_user = pkill_user.ok_or(Error::EnvVarNotSet(PKILL_USER_KEY))?;

    // get the gpio chip
    let mut chip =
        Chip::new(GPIOCHIP).with_context(|| format!("Can't read GPIO device: {}", GPIOCHIP))?;

    // configure Digital I/O pin to be used as mcu_running pin
    let line = chip
        .get_line(MCU_RUNNING_PIN_NUM)
        .with_context(|| format!("Can't get GPIO line {}", MCU_RUNNING_PIN_NUM))?;
    let handle = line
        .request(LineRequestFlags::OUTPUT, 1, GPIO_PIN_OWNER)
        .with_context(|| {
            format!(
                "Unable to create GPIO Line Handle {} as OUTPUT",
                MCU_RUNNING_PIN_NUM
            )
        })?;
    let mcu_running = CdevPin::new(handle).context("Unable to create CdevPin from Line Handle")?;

    // configure Digital I/O pin to be used as shutdown pin
    let line = chip
        .get_line(SHUTDOWN_PIN_NUM)
        .with_context(|| format!("Can't get GPIO line {}", SHUTDOWN_PIN_NUM))?;
    let handle = line
        .request(LineRequestFlags::INPUT, 0, GPIO_PIN_OWNER)
        .with_context(|| {
            format!(
                "Unable to create GPIO Line Handle {} as INPUT",
                SHUTDOWN_PIN_NUM
            )
        })?;
    let shutdown = CdevPin::new(handle).context("Unable to create CdevPin from Line Handle")?;

    // create the ChartPlotterHatPower device
    let mut cph = ChartPlotterHatPower::new(shutdown, mcu_running, Delay);

    // set the pin to show we are running
    cph.set_running()?;

    // now watch for shutdown signal
    loop {
        if cph.shutdown_signal()? {
            // run the pkill
            println!("Detected shutdown signal, powering off..");
            Command::new("/usr/bin/pkill")
                .arg("-u")
                .arg(pkill_user)
                .arg("opencpn")
                .output()
                .expect("failed to execute pkill");
            // sleep after the pkill to let opencpn quit gracefully
            thread::sleep(time::Duration::from_millis(pkill_delay * 1000));
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
