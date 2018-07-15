extern crate embedded_hal as hal;

use self::hal::blocking::delay::DelayUs;
use self::hal::digital::InputPin;
use self::hal::digital::OutputPin;

use BitDriver;
use Driver;
use Error;

pub trait OpenDrainOutput: OutputPin + InputPin {}
impl<P: OutputPin + InputPin> OpenDrainOutput for P {}

pub struct BlockingOpenDrainDriver<'a> {
    pin: &'a mut OpenDrainOutput,
    delay: &'a mut DelayUs<u16>,
}

impl<'a> BlockingOpenDrainDriver<'a> {
    pub fn new(
        pin: &'a mut OpenDrainOutput,
        delay: &'a mut DelayUs<u16>,
    ) -> BlockingOpenDrainDriver<'a> {
        BlockingOpenDrainDriver { pin, delay }
    }

    /// Allows the slave-device to change the
    /// state of the wire
    fn set_input(&mut self) {
        // high lets the pin 'float freely'
        // thus the slave-device can pull it
        // down or let it be pulled-up by the
        // pull-up resistor
        self.pin.set_high()
    }

    /// No longer allows the slave-device to
    /// change the state of the wire
    fn set_output(&mut self) {
        // no change required, on write_low()
        // the pin is pulled-down, on write_high()
        // it is let to be 'float freely' and thus
        // pulled-up by the pull-up resistor
    }

    /// Set the wire state to low
    fn write_low(&mut self) {
        // pull it down
        self.pin.set_low()
    }

    /// Set the write state to high
    fn write_high(&mut self) {
        // let it 'float freely' and thus
        // be pulled-up by the pull-up resistor
        self.pin.set_high()
    }

    /// Reads the current state of the wire
    fn read(&self) -> bool {
        self.pin.is_high()
    }

    /// Waits up to 250us for the wire to get high
    fn ensure_wire_high(&mut self) -> Result<(), Error> {
        for _ in 0..125 {
            if self.read_bit()? {
                return Ok(());
            }
            self.delay.delay_us(2);
        }
        Err(Error::WireNotHigh)
    }
}

impl<'a> Driver for BlockingOpenDrainDriver<'a> {
    fn reset(&mut self) -> Result<bool, Error> {
        // let mut cli = DisableInterrupts::new();
        self.set_input();
        // drop(cli);

        self.ensure_wire_high()?;
        // cli = DisableInterrupts::new();
        self.write_low();
        self.set_output();

        // drop(cli);
        self.delay.delay_us(480);
        // cli = DisableInterrupts::new();
        self.set_input();

        let mut val = false;
        for _ in 0..7 {
            self.delay.delay_us(10);
            val |= !self.read_bit()?;
        }
        // drop(cli);
        self.delay.delay_us(410);
        Ok(val)
    }
}

impl<'a> BitDriver for BlockingOpenDrainDriver<'a> {
    /// Somehow the theoretically timings do not work
    /// (tested on an stm32f103, bluepill). The tweaked
    /// timings below work.
    fn read_bit(&mut self) -> Result<bool, Error> {
        // let cli = DisableInterrupts::new();
        self.set_output();
        self.write_low();
        self.delay.delay_us(3);
        self.set_input();
        self.delay.delay_us(2); // was 10
        let val = self.read();
        // drop(cli);
        self.delay.delay_us(61); // was 53
        Ok(val)
    }

    /// Somehow the theoretically timings do not work
    /// (tested on an stm32f103, bluepill). The tweaked
    /// timings below work.
    fn write_bit(&mut self, high: bool) -> Result<(), Error> {
        // let cli = DisableInterrupts::new();
        self.write_low();
        self.set_output();
        self.delay.delay_us(if high { 10 } else { 65 });
        self.write_high();
        // drop(cli);
        self.delay.delay_us(if high { 55 } else { 5 });
        Ok(())
    }
}
