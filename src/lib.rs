#![no_std]
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub struct ShiftIn<L: OutputPin, C: OutputPin, I: InputPin, D: ShiftClockDelay, const N: usize> {
    latch: L,
    clock: C,
    input: I,
    delay: D,
}

pub enum Delay {
    LatchHigh,
    ClockLow,
    ClockHigh,
}

pub trait ShiftClockDelay {
    fn delay(&self, delay: Delay);
}

impl<L: OutputPin, C: OutputPin, I: InputPin, D: ShiftClockDelay, const N: usize>
    ShiftIn<L, C, I, D, N>
{
    /// Create a new shift in
    pub fn new(mut latch: L, mut clock: C, input: I, delay: D) -> Self {
        let _ = clock.set_low();
        let _ = latch.set_low();
        Self {
            latch,
            clock,
            input,
            delay,
        }
    }

    /// Read in all the data
    pub fn read(&mut self) -> [u8; N] {
        let mut data = [0; N];
        let _ = self.clock.set_low();
        let _ = self.latch.set_high();
        self.delay.delay(Delay::LatchHigh);
        let _ = self.latch.set_low();
        for byte in &mut data.iter_mut().rev() {
            for bit in (0..8).rev() {
                let _ = self.clock.set_low();
                self.delay.delay(Delay::ClockLow);
                if let Ok(h) = self.input.is_high() {
                    if h {
                        *byte = *byte | (1 << bit);
                    }
                }
                let _ = self.clock.set_high();
                self.delay.delay(Delay::ClockHigh);
            }
        }

        data
    }
}

impl<F> ShiftClockDelay for F
where
    F: Fn(Delay),
{
    fn delay(&self, d: Delay) {
        self(d)
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

    pub fn delayer(_d: Delay) {
        //do nothing
    }

    #[test]
    fn on_off() {
        let latch = Output;
        let clock = Output;
        let input = Input::new(0);

        let mut shift = ShiftIn::new(latch, clock, input, delayer);
        let res: [u8; 1] = shift.read();
        assert_eq!(res[0], 0);

        let latch = Output;
        let clock = Output;
        let input = Input::new(0xFFFF);

        let mut shift = ShiftIn::new(latch, clock, input, delayer);
        let mut res: [u8; 2] = shift.read();
        assert_eq!(res[0], 0xFF);
        assert_eq!(res[1], 0xFF);
        res = shift.read();
        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0);

        let latch = Output;
        let clock = Output;
        let input = Input::new(0xc2310);

        let mut shift = ShiftIn::new(latch, clock, input, delayer);
        let mut res: [u8; 2] = shift.read();
        assert_eq!(res[0], 0xc4);
        assert_eq!(res[1], 0x08);
        res = shift.read();
        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0x30);
        res = shift.read();
        assert_eq!(res[0], 0);
        assert_eq!(res[1], 0);
    }
}
