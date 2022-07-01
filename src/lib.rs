use embedded_hal::{
    blocking::spi::Transfer,
    digital::v2::{InputPin, OutputPin},
};
use std::{thread, time};

/// driver error
#[derive(Debug, PartialEq)]
pub enum PowerError<PE> {
    /// Pin Error
    PinError(PE),
}

/// driver error
#[derive(Debug, PartialEq)]
pub enum SpiError<SPIE, PE> {
    SpiError(SPIE),
    /// Pin Error
    PinError(PE),
}

/// device to work with ChartPlotterHat power functions
pub struct ChartPlotterHatPower<SP, RP> {
    shutdown_pin: SP,
    running_pin: RP,
}

impl<SP, RP, PE> ChartPlotterHatPower<SP, RP>
where
    SP: InputPin<Error = PE>,
    RP: OutputPin<Error = PE>,
{
    /// create a new `ChartPlotterHatPower` device
    pub fn new(shutdown_pin: SP, running_pin: RP) -> Self {
        ChartPlotterHatPower {
            shutdown_pin,
            running_pin,
        }
    }

    /// set the `running_pin` to high, indicating RaspberryPi is running
    pub fn set_running(&mut self) -> Result<(), PowerError<PE>> {
        self.running_pin.set_high().map_err(PowerError::PinError)?;
        Ok(())
    }

    /// check the `shutdown_pin`, to see if Chart Plotter Hat is signaling a shutdown
    pub fn shutdown_signal(&self) -> Result<bool, PowerError<PE>> {
        match self.shutdown_pin.is_high() {
            Ok(on) => {
                if on {
                    for _n in 0..6 {
                        thread::sleep(time::Duration::from_millis(100));
                        if !self.shutdown_pin.is_high().map_err(PowerError::PinError)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(PowerError::PinError(e)),
        }
    }
}

/// device to interact with ChartPlotterHat device SPI functions
pub struct ChartPlotterHatSpi<SPI, CS> {
    spi: SPI,
    cs: CS,
}

impl<SPI, CS, SPIE, PE> ChartPlotterHatSpi<SPI, CS>
where
    SPI: Transfer<u8, Error = SPIE>,
    CS: OutputPin<Error = PE>,
{
    /// create a new `ChartPlotterHatSpi` device
    pub fn new(spi: SPI, cs: CS) -> Self {
        ChartPlotterHatSpi { spi, cs }
    }

    /// method to send/retrieve data via SPI
    fn full_duplex(&mut self, addr: u8) -> Result<[u8; 2], SpiError<SPIE, PE>> {
        self.cs.set_low().map_err(SpiError::PinError)?;
        thread::sleep(time::Duration::from_micros(50));
        let mut tx_buf = [addr, 0x0, 0x0];
        for n in 0..3 {
            self.spi
                .transfer(&mut tx_buf[n..n + 1])
                .map_err(SpiError::SpiError)?;
            match n {
                0 => thread::sleep(time::Duration::from_micros(40)),
                1 => thread::sleep(time::Duration::from_micros(20)),
                _ => thread::sleep(time::Duration::from_micros(10)),
            }
        }
        self.cs.set_high().map_err(SpiError::PinError)?;
        Ok([tx_buf[1], tx_buf[2]])
    }

    /// get the version of the device
    pub fn get_version(&mut self) -> Result<[u8; 2], SpiError<SPIE, PE>> {
        self.full_duplex(0x04)
    }

    /// jump to bootloader of the avr
    pub fn enter_bootloader(&mut self) -> Result<(), SpiError<SPIE, PE>> {
        self.full_duplex(0x05)?;
        Ok(())
    }

    /// get whether device is can_enabled
    pub fn get_can_hardware(&mut self) -> Result<bool, SpiError<SPIE, PE>> {
        Ok(self.full_duplex(0x06)?[0] == 1)
    }

    /// toggle the eeprom pin on the device
    pub fn toggle_eeprom(&mut self) -> Result<(), SpiError<SPIE, PE>> {
        self.full_duplex(0x03)?;
        Ok(())
    }
}
