const TIMER_RATE: u64 = 60;

#[derive(Debug)]
pub struct Timer {
    value: u8,
    ticks_passed: u64,
    tick_limit: u64,
}

impl Timer {
    /// cpu rate -> X instructions per second
    pub fn with_cpu_rate(cpu_tick_rate: u64) -> Self {
        // limit = cpu_rate(1/s^2) * 1/60(1/s^2)
        let tick_limit = cpu_tick_rate / TIMER_RATE;
        Self {
            value: 0,
            ticks_passed: 0,
            tick_limit,
        }
    }

    pub fn tick(&mut self) {
        if self.value == 0 {
            return;
        }

        self.ticks_passed += 1;
        if self.ticks_passed >= self.tick_limit {
            self.value -= 1;
            self.ticks_passed = 0;
        }
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn set(&mut self, value: u8) {
        self.value = value;
        self.ticks_passed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_value_decreases_at_correct_rate() {
        let mut timer = Timer::with_cpu_rate(TIMER_RATE * 2);
        timer.set(1);

        timer.tick();
        assert_eq!(1, timer.get());

        timer.tick();
        assert_eq!(0, timer.get());
    }
}
