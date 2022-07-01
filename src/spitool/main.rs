use anyhow::{Context, Result};
use chartplotterhat_utils::ChartPlotterHatSpi;
use clap::Parser;
use embedded_hal::digital::v2::OutputPin;
use hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Spidev,
};
use linux_embedded_hal as hal;

// constants for devices
const GPIO_PIN_OWNER: &str = "spitool";
const SPIDEVNM: &str = "/dev/spidev0.0";
const GPIOCHIP: &str = "/dev/gpiochip0";

// raspberry pi pin numbers used by this executable
const CS_PIN_NUM: u32 = 8;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// toggle the eeprom
    #[clap(short, long, action)]
    eeprom: bool,
    /// verbose
    #[clap(short, long, action)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // configure SPI
    let mut spi =
        Spidev::open(SPIDEVNM).with_context(|| format!("Can't read device {}", SPIDEVNM))?;
    // set the SPI clk speed very slow, so spi slave can respond
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(32_000)
        .mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS)
        .build();
    spi.configure(&options)?;

    // configure Digital I/O pin to be used as cs pin
    let mut chip =
        Chip::new(GPIOCHIP).with_context(|| format!("Can't read GPIO device: {}", GPIOCHIP))?;
    let cs_line = chip
        .get_line(CS_PIN_NUM)
        .with_context(|| format!("Can't get GPIO line {}", CS_PIN_NUM))?;
    let cs_lh = cs_line
        .request(LineRequestFlags::OUTPUT, 1, GPIO_PIN_OWNER)
        .with_context(|| format!("Unable to create GPIO Line Handle {} as OUTPUT", CS_PIN_NUM))?;
    let mut cs = CdevPin::new(cs_lh)?;
    cs.set_high()?;

    // create the ChartPlotterhat device
    let mut cph = ChartPlotterHatSpi::new(spi, cs);
    let version = cph.get_version().unwrap();
    println!(
        "Chart Plotter Hat\n version: {}.{}\n can_hardware: {:?}",
        version[0],
        version[1],
        cph.get_can_hardware().unwrap()
    );

    if args.eeprom {
        if args.verbose {
            println!("Toggling the eeprom write line..");
        }
        cph.toggle_eeprom().unwrap();
    }
    Ok(())
}
