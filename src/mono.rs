// RTIC Monotonic impl for the 32-bit timers
use bsp::pac::{timer0, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
use fugit::{TimerDurationU32, TimerInstantU32};
use rtic::Monotonic;

pub struct MonoTimer<T: Instance32>(T);

impl<T: Instance32> MonoTimer<T> {
    pub fn new(timer: T) -> Self {
        timer.prescaler.write(
            |w| unsafe { w.prescaler().bits(4) }, // 1 MHz
        );
        timer.bitmode.write(|w| w.bitmode()._32bit());
        MonoTimer(timer)
    }
}

impl<T: Instance32> Monotonic for MonoTimer<T> {
    type Instant = TimerInstantU32<1_000_000>;
    type Duration = TimerDurationU32<1_000_000>;

    unsafe fn reset(&mut self) {
        self.0.intenset.modify(|_, w| w.compare0().set());
        self.0.tasks_clear.write(|w| w.bits(1));
        self.0.tasks_start.write(|w| w.bits(1));
    }

    #[inline(always)]
    fn now(&mut self) -> Self::Instant {
        self.0.tasks_capture[1].write(|w| unsafe { w.bits(1) });
        Self::Instant::from_ticks(self.0.cc[1].read().bits())
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        self.0.cc[0].write(|w| unsafe { w.cc().bits(instant.duration_since_epoch().ticks()) });
    }

    fn clear_compare_flag(&mut self) {
        self.0.events_compare[0].write(|w| w);
    }

    #[inline(always)]
    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }
}

pub trait Instance32: core::ops::Deref<Target = timer0::RegisterBlock> {}
impl Instance32 for TIMER0 {}
impl Instance32 for TIMER1 {}
impl Instance32 for TIMER2 {}
impl Instance32 for TIMER3 {}
impl Instance32 for TIMER4 {}