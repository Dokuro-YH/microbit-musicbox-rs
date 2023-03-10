use bsp::hal::gpio::{Input, Pin, PullUp};
use bsp::hal::prelude::*;
use defmt::Format;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Pending = 0,
    Down = 1,
    Up = 2,
    Count = 3,
    Press = 6,
    Pressend = 7,
}

pub struct Button<const TIMER_HZ: u32> {
    pin: Pin<Input<PullUp>>,
    state: State,
    last_state: State,
    cnt_click: u32,
    attach_event_fn: Option<fn(Event)>,
    time: TimerInstantU64<TIMER_HZ>,
    start_time: TimerInstantU64<TIMER_HZ>,
    debounce_ms: TimerDurationU64<TIMER_HZ>,
    click_ms: TimerDurationU64<TIMER_HZ>,
    press_ms: TimerDurationU64<TIMER_HZ>,
}

impl<const TIMER_HZ: u32> Button<TIMER_HZ> {
    pub fn new(pin: Pin<Input<PullUp>>) -> Self {
        Self {
            pin,
            state: State::Pending,
            last_state: State::Pending,
            cnt_click: 0,
            attach_event_fn: None,
            time: TimerInstantU64::from_ticks(0),
            start_time: TimerInstantU64::from_ticks(0),
            debounce_ms: 50.millis(),
            click_ms: 400.millis(),
            press_ms: 800.millis(),
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

    pub fn free(self) -> Pin<Input<PullUp>> {
        self.pin
    }

    pub fn reset(&mut self) {
        self.state = State::Pending;
        self.last_state = State::Pending;
        self.cnt_click = 0;
        self.time = TimerInstantU64::from_ticks(0);
        self.start_time = TimerInstantU64::from_ticks(0);
    }

    pub fn tick(&mut self) {
        let active = self.pin.is_low().unwrap();
        let now = self.now();
        let wait_time = now - self.start_time;

        use State::*;
        match self.state {
            Pending => {
                if active {
                    self.update_state(Down);
                    self.cnt_click = 0;
                    self.start_time = now;
                }
            }
            Down => {
                if !active && (wait_time < self.debounce_ms) {
                    self.update_state(self.last_state);
                } else if !active {
                    self.update_state(Up);
                } else if active && (wait_time > self.press_ms) {
                    self.update_state(Press);
                    if let Some(f) = self.attach_event_fn {
                        f(Event::LongPressStart)
                    }
                }
            }
            Up => {
                if active && (wait_time < self.debounce_ms) {
                    self.update_state(self.last_state);
                } else if wait_time >= self.debounce_ms {
                    self.cnt_click += 1;
                    self.update_state(Count);
                }
            }
            Count => {
                if active {
                    self.update_state(Down);
                    self.start_time = now;
                } else if wait_time > self.click_ms {
                    if let Some(f) = self.attach_event_fn {
                        match self.cnt_click {
                            1 => f(Event::Click),
                            2 => f(Event::DoubleClick),
                            cnt => f(Event::MultiClick(cnt)),
                        }
                    }
                    self.reset();
                }
            }
            Press => {
                if !active {
                    self.update_state(Pressend);
                    self.start_time = now;
                } else if let Some(f) = self.attach_event_fn {
                    f(Event::LongPressDuring)
                }
            }
            Pressend => {
                if active && (wait_time < self.debounce_ms) {
                    self.update_state(self.last_state);
                } else if wait_time > self.debounce_ms {
                    if let Some(f) = self.attach_event_fn {
                        f(Event::LongPressStop)
                    }
                    self.reset();
                }
            }
        }
    }

    fn now(&mut self) -> TimerInstantU64<TIMER_HZ> {
        if self.state != State::Pending {
            self.time += TimerDurationU64::from_ticks(1);
        }
        self.time
    }

    fn update_state(&mut self, state: State) {
        self.last_state = self.state;
        self.state = state;
    }
}
