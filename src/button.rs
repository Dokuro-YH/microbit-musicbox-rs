use core::fmt::Debug;

use defmt::Format;
use embedded_hal::digital::InputPin;
use fugit::{ExtU64, TimerDurationU64, TimerInstantU64};

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Click,
    DoubleClick,
    MultiClick(u32),
    LongPressStart,
    LongPressDuring,
    LongPressStop,
}

pub struct Button<PIN, const TIMER_HZ: u32> {
    pin: PIN,
    state: State,
    last_state: State,
    cnt_click: u32,
    attach_event_fn: Option<fn(Event)>,
    start_time: TimerInstantU64<TIMER_HZ>,
    debounce_ms: TimerDurationU64<TIMER_HZ>,
    click_ms: TimerDurationU64<TIMER_HZ>,
    press_ms: TimerDurationU64<TIMER_HZ>,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
enum State {
    Pending = 0,
    Down = 1,
    Up = 2,
    Count = 3,
    Press = 6,
    Pressend = 7,
}

impl<PIN, E, const TIMER_HZ: u32> Button<PIN, TIMER_HZ>
where
    E: Debug,
    PIN: InputPin<Error = E>,
{
    pub fn new(pin: PIN) -> Self {
        Self {
            pin,
            state: State::Pending,
            last_state: State::Pending,
            cnt_click: 0,
            attach_event_fn: None,
            start_time: TimerInstantU64::from_ticks(0),
            debounce_ms: 50.millis(),
            click_ms: 200.millis(),
            press_ms: 500.millis(),
        }
    }

    pub fn set_debounce_ms(&mut self, debounce_ms: TimerDurationU64<TIMER_HZ>) {
        self.debounce_ms = debounce_ms;
    }

    pub fn set_click_ms(&mut self, click_ms: TimerDurationU64<TIMER_HZ>) {
        self.click_ms = click_ms;
    }

    pub fn set_press_ms(&mut self, press_ms: TimerDurationU64<TIMER_HZ>) {
        self.press_ms = press_ms;
    }

    pub fn attach_event(&mut self, f: fn(Event)) {
        self.attach_event_fn = Some(f);
    }

    pub fn free(self) -> PIN {
        self.pin
    }

    pub fn tick(&mut self, time: &TimerInstantU64<TIMER_HZ>) {
        use State::*;

        let active = self.pin.is_low().unwrap();
        let wait_time = *time - self.start_time;

        match self.state {
            Pending => {
                if active {
                    self.update_state(Down);
                    self.cnt_click = 0;
                    self.start_time = *time;
                }
            }
            Down => {
                if !active && wait_time > self.debounce_ms {
                    self.update_state(Up);
                } else if active && wait_time > self.press_ms {
                    self.update_state(Press);
                    self.attach_event_fn.map(|f| f(Event::LongPressStart));
                }
            }
            Up => {
                if !active && wait_time > self.debounce_ms {
                    self.cnt_click += 1;
                    self.update_state(Count);
                }
            }
            Count => {
                if active {
                    self.update_state(Down);
                    self.start_time = *time;
                } else if wait_time > self.click_ms {
                    self.attach_event_fn.map(|f| {
                        f(match self.cnt_click {
                            1 => Event::Click,
                            2 => Event::DoubleClick,
                            cnt => Event::MultiClick(cnt),
                        })
                    });
                    self.reset();
                }
            }
            Press => {
                if !active {
                    self.update_state(Pressend);
                    self.start_time = *time;
                } else {
                    self.attach_event_fn.map(|f| f(Event::LongPressDuring));
                }
            }
            Pressend => {
                if !active && wait_time > self.debounce_ms {
                    self.attach_event_fn.map(|f| f(Event::LongPressStop));
                    self.reset();
                }
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.state = State::Pending;
        self.last_state = State::Pending;
        self.cnt_click = 0;
        self.start_time = TimerInstantU64::from_ticks(0);
    }

    #[inline]
    fn update_state(&mut self, state: State) {
        self.last_state = self.state;
        self.state = state;
    }
}
