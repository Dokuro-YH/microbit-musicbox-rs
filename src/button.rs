use core::fmt::Debug;

use embedded_hal::digital::InputPin;
use fugit::{TimerDurationU64, TimerInstantU64};

use musicbox::button_detector::ButtonDetector;
pub use musicbox::button_detector::ButtonEvent as Event;

/// Imperative Shell：负责 GPIO 读取、时间转换和回调调度。
/// 按钮检测的核心状态机委托给功能核心 [`ButtonDetector`]。
pub struct Button<PIN, const TIMER_HZ: u32> {
    pin: PIN,
    detector: ButtonDetector,
    attach_event_fn: Option<fn(Event)>,
}

impl<PIN, E, const TIMER_HZ: u32> Button<PIN, TIMER_HZ>
where
    E: Debug,
    PIN: InputPin<Error = E>,
{
    pub fn new(pin: PIN) -> Self {
        Self {
            pin,
            detector: ButtonDetector::new(),
            attach_event_fn: None,
        }
    }

    pub fn set_debounce_ms(&mut self, debounce_ms: TimerDurationU64<TIMER_HZ>) {
        self.detector.debounce_ms = debounce_ms.ticks() / (TIMER_HZ as u64 / 1000);
    }

    pub fn set_click_ms(&mut self, click_ms: TimerDurationU64<TIMER_HZ>) {
        self.detector.click_ms = click_ms.ticks() / (TIMER_HZ as u64 / 1000);
    }

    pub fn set_press_ms(&mut self, press_ms: TimerDurationU64<TIMER_HZ>) {
        self.detector.press_ms = press_ms.ticks() / (TIMER_HZ as u64 / 1000);
    }

    pub fn attach_event(&mut self, f: fn(Event)) {
        self.attach_event_fn = Some(f);
    }

    pub fn free(self) -> PIN {
        self.pin
    }

    pub fn tick(&mut self, time: &TimerInstantU64<TIMER_HZ>) {
        let active = self.pin.is_low().unwrap();
        let now_ms = time.duration_since_epoch().ticks() / (TIMER_HZ as u64 / 1000);

        if let Some(event) = self.detector.update(now_ms, active) {
            if let Some(f) = self.attach_event_fn {
                f(event);
            }
        }
    }
}
