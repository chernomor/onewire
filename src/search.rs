
use super::ADDRESS_BITS;
use super::ADDRESS_BYTES;


#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SearchState {
    Initialized,
    DeviceFound,
    End,
}

#[derive(Clone)]
pub struct DeviceSearch {
    pub(crate) address:       [u8; 8],
    pub(crate) discrepancies: [u8; 8],
    pub(crate) state: SearchState,
}

impl DeviceSearch {
    pub fn new() -> DeviceSearch {
        DeviceSearch {
            address:       [0u8; ADDRESS_BYTES as usize],
            discrepancies: [0u8; ADDRESS_BYTES as usize],
            state:         SearchState::Initialized,
        }
    }

    pub fn new_for_family(family: u8) -> DeviceSearch {
        let mut search = DeviceSearch::new();
        search.address[0] = family;
        search
    }

    pub(crate) fn is_bit_set_in_address(&self, bit: u8) -> bool {
        DeviceSearch::is_bit_set(&self.address, bit)
    }

    pub(crate) fn set_bit_in_address(&mut self, bit: u8) {
        DeviceSearch::set_bit(&mut self.address, bit);
    }

    pub(crate) fn reset_bit_in_address(&mut self, bit: u8) {
        DeviceSearch::reset_bit(&mut self.address, bit);
    }

    pub(crate)fn write_bit_in_address(&mut self, bit: u8, value: bool) {
        if value {
            self.set_bit_in_address(bit);
        } else {
            self.reset_bit_in_address(bit);
        }
    }

    pub(crate)fn is_bit_set_in_discrepancies(&self, bit: u8) -> bool {
        DeviceSearch::is_bit_set(&self.discrepancies, bit)
    }

    pub(crate)fn set_bit_in_discrepancy(&mut self, bit: u8) {
        DeviceSearch::set_bit(&mut self.discrepancies, bit);
    }

    pub(crate)fn reset_bit_in_discrepancy(&mut self, bit: u8) {
        DeviceSearch::reset_bit(&mut self.discrepancies, bit);
    }

    pub(crate)fn write_bit_in_discrepancy(&mut self, bit: u8, value: bool) {
        if value {
            self.set_bit_in_discrepancy(bit);
        } else {
            self.reset_bit_in_discrepancy(bit);
        }
    }

    fn is_bit_set(array: &[u8], bit: u8) -> bool {
        if bit / 8 >= array.len() as u8 {
            return false;
        }
        let index = bit / 8;
        let offset = bit % 8;
        array[index as usize] & (0x01 << offset) != 0x00
    }

    fn set_bit(array: &mut [u8], bit: u8) {
        if bit / 8 >= array.len() as u8 {
            return;
        }
        let index = bit / 8;
        let offset = bit % 8;
        array[index as usize] |= 0x01 << offset
    }

    fn reset_bit(array: &mut [u8], bit: u8) {
        if bit / 8 >= array.len() as u8 {
            return;
        }
        let index = bit / 8;
        let offset = bit % 8;
        array[index as usize] &= !(0x01 << offset)
    }

    pub fn last_discrepancy(&self) -> Option<u8> {
        let mut result = None;
        for i in 0..ADDRESS_BITS {
            if self.is_bit_set_in_discrepancies(i) {
                result = Some(i);
            }
        }
        result
    }
}