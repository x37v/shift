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
    use embedded_hal::digital::v2::{InputPin, OutputPin};

    struct Input {
        high: bool,
    }
    struct Output;

    impl InputPin for Input {
        type Error = ();
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.high)
        }
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(!self.high)
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
        let input = Input { high: false };

        let mut shift = ShiftIn::new(latch, clock, input, delayer);
        let res: [u8; 1] = shift.read();
        assert_eq!(res[0], 0);

        let latch = Output;
        let clock = Output;
        let input = Input { high: true };

        let mut shift = ShiftIn::new(latch, clock, input, delayer);
        let res: [u8; 2] = shift.read();
        assert_eq!(res[0], 0xFF);
        assert_eq!(res[1], 0xFF);
    }
}