#![no_std]
use embedded_hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};

pub struct ShiftIn<
    L: OutputPin,
    C: OutputPin,
    I: InputPin,
    const DELAY_LATCH_HIGH: u8,
    const DELAY_CLOCK_LOW: u8,
    const DELAY_CLOCK_HIGH: u8,
> {
    latch: L,
    clock: C,
    input: I,
}

impl<
        L: OutputPin,
        C: OutputPin,
        I: InputPin,
        const DELAY_LATCH_HIGH: u8,
        const DELAY_CLOCK_LOW: u8,
        const DELAY_CLOCK_HIGH: u8,
    > ShiftIn<L, C, I, DELAY_LATCH_HIGH, DELAY_CLOCK_LOW, DELAY_CLOCK_HIGH>
{
    /// Create a new shift in
    pub fn new(mut latch: L, mut clock: C, input: I) -> Self {
        let _ = clock.set_low();
        let _ = latch.set_low();
        Self {
            latch,
            clock,
            input,
        }
    }

    /// Read in all the data
    pub fn read<const N: usize>(&mut self, delay: &mut dyn DelayUs<u8>) -> [u8; N] {
        let mut data = [0; N];
        let _ = self.clock.set_low();
        let _ = self.latch.set_high();
        delay.delay_us(DELAY_LATCH_HIGH);
        let _ = self.latch.set_low();
        for byte in &mut data.iter_mut().rev() {
            for bit in (0..8).rev() {
                let _ = self.clock.set_low();
                delay.delay_us(DELAY_CLOCK_LOW);
                if let Ok(h) = self.input.is_high() {
                    if h {
                        *byte = *byte | (1 << bit);
                    }
                }
                let _ = self.clock.set_high();
                delay.delay_us(DELAY_CLOCK_HIGH);
            }
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use embedded_hal::digital::v2::{InputPin, OutputPin};

    struct Input {
        pattern: usize,
        index: AtomicUsize,
    }
    struct Output;

    impl Input {
        pub fn new(pattern: usize) -> Self {
            Self {
                pattern,
                index: AtomicUsize::new(0),
            }
        }
    }

    impl InputPin for Input {
        type Error = ();
        fn is_high(&self) -> Result<bool, Self::Error> {
            let index = self.index.fetch_add(1, Ordering::SeqCst);
            Ok(self.pattern & (1 << index) != 0)
        }
        fn is_low(&self) -> Result<bool, Self::Error> {
            let index = self.index.fetch_add(1, Ordering::SeqCst);
            Ok(self.pattern & (1 << index) == 0)
        }
    }

    impl OutputPin for Output {
        type Error = ();
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct FakeDelay;

    impl embedded_hal::blocking::delay::DelayUs<u8> for FakeDelay {
        fn delay_us(&mut self, _v: u8) {
            //do nothing
        }
    }

    #[test]
    fn on_off() {
        let latch = Output;
        let clock = Output;
        let input = Input::new(0);
        let mut delay = FakeDelay;

        let mut shift: ShiftIn<_, _, _, 1, 1, 1> = ShiftIn::new(latch, clock, input);
        let res: [u8; 1] = shift.read(&mut delay);
        assert_eq!(res[0], 0);

        let latch = Output;
        let clock = Output;
        let input = Input::new(0xFFFF);

        let mut shift: ShiftIn<_, _, _, 1, 1, 1> = ShiftIn::new(latch, clock, input);
        let mut res: [u8; 2] = shift.read(&mut delay);
        assert_eq!(res[0], 0xFF);
        assert_eq!(res[1], 0xFF);
        res = shift.read(&mut delay);
        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0);

        let latch = Output;
        let clock = Output;
        let input = Input::new(0xc2310);

        let mut shift: ShiftIn<_, _, _, 1, 1, 1> = ShiftIn::new(latch, clock, input);
        let mut res: [u8; 2] = shift.read(&mut delay);
        assert_eq!(res[0], 0xc4);
        assert_eq!(res[1], 0x08);
        res = shift.read(&mut delay);
        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0x30);
        res = shift.read(&mut delay);
        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0);
    }
}
