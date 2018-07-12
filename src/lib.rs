#![feature(asm)]

#![no_std]
#![crate_name = "onewire"]

extern crate byteorder;

pub mod ds18b20;

pub use ds18b20::DS18B20;

pub const ADDRESS_BYTES : u8 = 8;
pub const ADDRESS_BITS  : u8 = ADDRESS_BYTES * 8;

pub mod driver;
pub use driver::*;

pub mod device;
pub use device::*;

pub mod search;
pub use search::*;

#[repr(u8)]
pub enum Command {
    SelectRom = 0x55,
    SearchNext = 0xF0,
    SearchNextAlarmed = 0xEC,
}

#[derive(Debug)]
pub enum Error {
    WireNotHigh,
    CrcMismatch(u8, u8),
    FamilyCodeMismatch(u8, u8),
    Debug(Option<u8>),
}


pub struct OneWire<D: ByteDriver> {
    driver: D,
}

impl<D: ByteDriver> OneWire<D> {
    pub fn new(driver: D) -> OneWire<D> {
        OneWire {
            driver,
        }
    }

    pub fn reset_select_write_read(&mut self, device: &Device, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        self.driver.reset()?;
        self.select(device)?;
        self.driver.write_bytes(write)?;
        self.driver.read_bytes(read)?;
        Ok(())
    }

    pub fn reset_select_read_only(&mut self, device: &Device, read: &mut [u8]) -> Result<(), Error> {
        self.driver.reset()?;
        self.select(device)?;
        self.driver.read_bytes(read)?;
        Ok(())
    }

    pub fn reset_select_write_only(&mut self, device: &Device, write: &[u8]) -> Result<(), Error> {
        self.driver.reset()?;
        self.select(device)?;
        self.driver.write_bytes(write)?;
        Ok(())
    }

    pub fn select(&mut self, device: &Device) -> Result<(), Error> {
        self.write_command(Command::SelectRom)?;
        self.driver.write_bytes(&device.address)
    }

    pub fn search_next(&mut self, search: &mut DeviceSearch) -> Result<Option<Device>, Error> where D: BitDriver {
        self.search(search, Command::SearchNext)
    }

    pub fn search_next_alarmed(&mut self, search: &mut DeviceSearch) -> Result<Option<Device>, Error> where D: BitDriver {
        self.search(search, Command::SearchNextAlarmed)
    }

    /// Heavily inspired by https://github.com/ntruchsess/arduino-OneWire/blob/85d1aae63ea4919c64151e03f7e24c2efbc40198/OneWire.cpp#L362
    fn search(&mut self, rom: &mut DeviceSearch, command: Command) -> Result<Option<Device>, Error> where D: BitDriver {
        if SearchState::End == rom.state {
            return Ok(None);
        }

        let mut discrepancy_found = false;
        let last_discrepancy = rom.last_discrepancy();

        if !self.driver.reset()? {
            return Ok(None);
        }

        self.write_command(command)?;

        if let Some(last_discrepancy) = last_discrepancy {
            // walk previous path
            for i in 0..last_discrepancy {
                let bit0 = self.driver.read_bit()?;
                let bit1 = self.driver.read_bit()?;

                if bit0 && bit1 {
                    // no device responded
                    return Ok(None);

                } else {
                    let bit = rom.is_bit_set_in_address(i);
                    // rom.write_bit_in_address(i, bit0);
                    // rom.write_bit_in_discrepancy(i, bit);
                    self.driver.write_bit(bit)?;
                }
            }
        } else {
            // no discrepancy and device found, meaning the one found is the only one
            if rom.state == SearchState::DeviceFound {
                rom.state = SearchState::End;
                return Ok(None);
            }
        }

        for i in last_discrepancy.unwrap_or(0)..ADDRESS_BITS {
            let bit0 = self.driver.read_bit()?; // normal bit
            let bit1 = self.driver.read_bit()?; // complementar bit

            if last_discrepancy.eq(&Some(i)) {
                // be sure to go different path from before (go second path, thus writing 1)
                rom.reset_bit_in_discrepancy(i);
                rom.set_bit_in_address(i);
                self.driver.write_bit(true)?;

            } else {
                if bit0 && bit1 {
                    // no response received
                    return Ok(None);
                }

                if !bit0 && !bit1 {
                    // addresses with 0 and 1
                    // found new path, go first path by default (thus writing 0)
                    discrepancy_found |= true;
                    rom.set_bit_in_discrepancy(i);
                    rom.reset_bit_in_address(i);
                    self.driver.write_bit(false)?;

                } else {
                    // addresses only with bit0
                    rom.write_bit_in_address(i, bit0);
                    self.driver.write_bit(bit0)?;
                }
            }
        }

        if !discrepancy_found && rom.last_discrepancy().is_none() {
            rom.state = SearchState::End;
        } else {
            rom.state = SearchState::DeviceFound;
        }
        Ok(Some(Device {
            address: rom.address.clone()
        }))
    }


    fn write_command(&mut self, command: Command) -> Result<(), Error> {
        self.driver.write_byte(command as u8)
    }
}


pub fn ensure_correct_rcr8(device: &Device, data: &[u8], crc8: u8) -> Result<(), Error> {
    let computed = compute_crc8(device, data);
    if computed != crc8 {
        Err(Error::CrcMismatch(computed, crc8))
    } else {
        Ok(())
    }
}

pub fn compute_crc8(device: &Device, data: &[u8]) -> u8 {
    let crc = compute_partial_crc8(0u8, &device.address[..]);
    compute_partial_crc8(crc, data)
}

pub fn compute_partial_crc8(crc: u8, data: &[u8]) -> u8 {
    let mut crc = crc;
    for byte in data.iter() {
        let mut byte = *byte;
        for _ in 0..8 {
            let mix = (crc ^ byte) & 0x01;
            crc >>= 1;
            if mix != 0x00 {
                crc ^= 0x8C;
            }
            byte >>= 1;
        }
    }
    crc
}