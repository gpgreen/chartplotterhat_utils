use chartplotterhat_utils::ChartPlotterHatSpi;
use clap::Parser;
use embedded_hal::digital::v2::OutputPin;
use hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Spidev,
};
use linux_embedded_hal as hal;

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

fn main() {
    let args = Args::parse();

    // configure SPI
    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    // set the SPI clk speed very slow, so spi slave can respond
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(32_000)
        .mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS)
        .build();
    spi.configure(&options).unwrap();

    // configure Digital I/O pin to be used as cs pin
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let cs_line = chip.get_line(8).unwrap();
    let cs_lh = cs_line
        .request(LineRequestFlags::OUTPUT, 1, "spitool")
        .unwrap();
    let mut cs = CdevPin::new(cs_lh).unwrap();
    cs.set_high().unwrap();

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
}
