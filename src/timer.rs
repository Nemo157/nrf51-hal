use core::time::Duration;

use nb::{Result, Error};
use hal::timer::{CountDown, Periodic};
use nrf51::TIMER0;

pub struct Timer(TIMER0);

impl Timer {
    pub fn new(timer: TIMER0) -> Timer {
        Timer(timer)
    }
}

impl CountDown for Timer {
    type Time = Duration;

    fn start<T>(&mut self, count: T) where T: Into<Self::Time> {
        let duration = count.into();
        let us = (duration.as_secs() as u32) * 1_000_000 + duration.subsec_micros();

        self.0.bitmode.write(|w| w
            .bitmode()._32bit());

        self.0.intenset.write(|w| w
            .compare0().set());

        self.0.cc[0].write(|w| unsafe { w.bits(us) });

        self.0.events_compare[0].reset();
        self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

    }

    fn wait(&mut self) -> Result<(), !> {
        if self.0.events_compare[0].read().bits() == 1 {
            self.0.events_compare[0].reset();
            self.0.tasks_clear.write(|w| unsafe { w.bits(1) });
            Ok(())
        } else {
            Err(Error::WouldBlock)
        }
    }
}

impl Periodic for Timer {
}
