use super::ADDRESS_BYTES;

use core::fmt::Display;
use core::fmt::Formatter;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Device {
    pub address: [u8; ADDRESS_BYTES as usize],
}

impl Device {
    pub fn family_code(&self) -> u8 {
        self.address[0]
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter) -> ::core::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
            self.address[6],
            self.address[7],
        )
    }
}
