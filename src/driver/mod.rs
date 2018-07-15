
#[cfg(feature = "embedded-opendrain")] mod embedded_opendrain;
#[cfg(feature = "embedded-opendrain")] pub use self::embedded_opendrain::*;

#[cfg(feature = "embedded-spi")] mod embedded_spi;
#[cfg(feature = "embedded-spi")] pub use self::embedded_spi::*;

#[cfg(feature = "linux-gpio")] mod linux_gpio;
#[cfg(feature = "linux-gpio")] pub use self::linux_gpio::*;


use super::Error;

pub trait Driver {
    /// Performs a reset and listens for a presence pulse
    /// Returns Err(WireNotHigh) if the wire seems to be shortened,
    /// Ok(true) if presence pulse has been received and Ok(false)
    /// if no other device was detected but the wire seems to be ok
    ///
    ///
    ///    Reset procedure
    ///
    /// A
    /// |         +-????---------
    /// |         | ????
    /// |---------+ ????
    /// +---------------------------> µs
    /// 0        480  |         960
    ///              550
    ///      Presence pulse: low if there is a slave-device
    ///
    fn reset(&mut self) -> Result<bool, Error>;
}

/// See https://www.maximintegrated.com/en/app-notes/index.mvp/id/126
pub trait BitDriver: Driver {

    ///    Read low bit
    ///
    /// A
    /// |   ???               +-
    /// |   ???               |
    /// |---???---------------+
    /// +---------------------------> µs
    /// 0   6   15           70
    ///          |
    ///      Recommended read here
    ///
    ///
    ///
    ///    Read high bit
    ///
    /// A
    /// |   ???-----------------
    /// |   ???
    /// |---???
    /// +---------------------------> µs
    /// 0   6   15           70
    ///          |
    ///      Recommended read here
    ///
    fn read_bit(&mut self) -> Result<bool, Error>;

    ///    Write low bit
    ///
    /// A
    /// |                  +----
    /// |                  |
    /// |------------------+
    /// +---------------------------> µs
    /// 0                 60 70
    ///
    ///
    ///    Write high bit
    ///
    /// A
    /// |   +-------------------
    /// |   |
    /// |---+
    /// +---------------------------> µs
    /// 0   6                70
    ///
    ///
    fn write_bit(&mut self, data: bool) -> Result<(), Error>;
}

pub trait ByteDriver: Driver {
    fn read_byte(&mut self) -> Result<u8, Error>;

    fn read_bytes(&mut self, data: &mut [u8]) -> Result<(), Error> {
        for byte in data.iter_mut() {
            *byte = self.read_byte()?;
        }
        Ok(())
    }

    fn write_byte(&mut self, data: u8) -> Result<(), Error>;

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Error> {
        for byte in data {
            self.write_byte(*byte)?;
        }
        Ok(())
    }
}

impl<T: BitDriver> ByteDriver for T {
    fn read_byte(&mut self) -> Result<u8, Error> {
        fn to_bit(state: bool) -> u8 {
            if state {
                0x01
            } else {
                0x00
            }
        }
        Ok(
            to_bit(self.read_bit()?) << 7
                | to_bit(self.read_bit()?) << 6
                | to_bit(self.read_bit()?) << 5
                | to_bit(self.read_bit()?) << 4
                | to_bit(self.read_bit()?) << 3
                | to_bit(self.read_bit()?) << 2
                | to_bit(self.read_bit()?) << 1
                | to_bit(self.read_bit()?)
        )
    }

    fn write_byte(&mut self, data: u8) -> Result<(), Error> {
        fn is_bit_set(byte: u8, n: u8) -> bool {
            let flag = 0x01 << n;
            byte & flag == flag
        }
        self.write_bit(is_bit_set(data, 7))?;
        self.write_bit(is_bit_set(data, 6))?;
        self.write_bit(is_bit_set(data, 5))?;
        self.write_bit(is_bit_set(data, 4))?;
        self.write_bit(is_bit_set(data, 3))?;
        self.write_bit(is_bit_set(data, 2))?;
        self.write_bit(is_bit_set(data, 1))?;
        self.write_bit(is_bit_set(data, 0))?;
        Ok(())
    }
}