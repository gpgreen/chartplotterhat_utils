use embedded_hal::{
    blocking::{delay::DelayMs, delay::DelayUs, spi::Transfer},
    digital::v2::{InputPin, OutputPin},
};
use thiserror::Error;

/// SPI driver error
#[derive(Error, Debug, PartialEq)]
pub enum SpiError<SPIE, PE> {
    /// SPI error
    #[error("transparent")]
    Spi(#[from] SPIE),
    /// Pin Error
    #[error("transparent")]
    Pin(PE),
    #[error("ADC Channel is not operating")]
    BadChannel,
}

/// device to work with ChartPlotterHat power functions
pub struct ChartPlotterHatPower<SP, RP, D> {
    shutdown_pin: SP,
    running_pin: RP,
    timer: D,
}

impl<SP, RP, D, PE> ChartPlotterHatPower<SP, RP, D>
where
    SP: InputPin<Error = PE>,
    RP: OutputPin<Error = PE>,
    D: DelayMs<u32>,
{
    /// create a new `ChartPlotterHatPower` device
    pub fn new(shutdown_pin: SP, running_pin: RP, timer: D) -> Self {
        ChartPlotterHatPower {
            shutdown_pin,
            running_pin,
            timer,
        }
    }

    /// set the `running_pin` to high, indicating RaspberryPi is running
    pub fn set_running(&mut self) -> Result<(), PE> {
        self.running_pin.set_high()?;
        Ok(())
    }

    /// check the `shutdown_pin`, to see if the hat is signaling a shutdown
    /// if signaled, loop the required amount of time to verify that it stays signaled
    /// return the result, true if signaled correctly, false if no
    pub fn shutdown_signal(&mut self) -> Result<bool, PE> {
        match self.shutdown_pin.is_high() {
            Ok(on) => {
                if on {
                    for _n in 0..6 {
                        self.timer.delay_ms(100);
                        if !self.shutdown_pin.is_high()? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// device to interact with ChartPlotterHat device SPI functions
pub struct ChartPlotterHatSpi<SPI, CS, D> {
    spi: SPI,
    cs: CS,
    channels: u8,
    timer: D,
}

impl<SPI, CS, D, SPIE, PE> ChartPlotterHatSpi<SPI, CS, D>
where
    SPI: Transfer<u8, Error = SPIE>,
    CS: OutputPin<Error = PE>,
    D: DelayUs<u32>,
{
    /// create a new `ChartPlotterHatSpi` device
    pub fn new(spi: SPI, cs: CS, timer: D) -> Self {
        ChartPlotterHatSpi {
            spi,
            cs,
            channels: 0,
            timer,
        }
    }

    /// method to send/retrieve data via SPI
    fn full_duplex(&mut self, addr: u8, data: u8) -> Result<[u8; 2], SpiError<SPIE, PE>> {
        self.cs.set_low().map_err(SpiError::Pin)?;
        self.timer.delay_us(50);
        let mut tx_buf = [addr, data, 0x0];
        for n in 0..3 {
            self.spi.transfer(&mut tx_buf[n..n + 1])?;
            match n {
                0 => self.timer.delay_us(40),
                1 => self.timer.delay_us(20),
                _ => self.timer.delay_us(10),
            }
        }
        self.cs.set_high().map_err(SpiError::Pin)?;
        Ok([tx_buf[1], tx_buf[2]])
    }

    /// get the version of the device
    pub fn get_version(&mut self) -> Result<[u8; 2], SpiError<SPIE, PE>> {
        self.full_duplex(0x04, 0)
    }

    /// jump to bootloader of the avr
    pub fn enter_bootloader(&mut self) -> Result<(), SpiError<SPIE, PE>> {
        self.full_duplex(0x05, 0)?;
        Ok(())
    }

    /// get whether device is can_enabled
    pub fn get_can_hardware(&mut self) -> Result<bool, SpiError<SPIE, PE>> {
        Ok(self.full_duplex(0x06, 0)?[0] == 1)
    }

    /// toggle the eeprom pin on the device
    pub fn toggle_eeprom(&mut self) -> Result<(), SpiError<SPIE, PE>> {
        self.full_duplex(0x03, 0)?;
        Ok(())
    }

    /// set which adc channel to read
    pub fn set_channel(&mut self, ch: u8) -> Result<(), SpiError<SPIE, PE>> {
        if ch > 4 {
            return Err(SpiError::BadChannel);
        }
        self.channels |= 1 << ch;
        self.full_duplex(0x01, self.channels)?;
        Ok(())
    }

    /// get which channels are operating
    pub fn get_channels(&mut self) -> Result<u8, SpiError<SPIE, PE>> {
        let channels = self.full_duplex(0x02, 0)?;
        Ok(channels[0])
    }

    /// get a reading from operating channel
    pub fn get_reading(&mut self, ch: u8) -> Result<u16, SpiError<SPIE, PE>> {
        if ch > 4 {
            return Err(SpiError::BadChannel);
        }
        let result = self.full_duplex(0x10 + ch, 0)?;
        Ok(u16::from_be_bytes(result))
    }
}
