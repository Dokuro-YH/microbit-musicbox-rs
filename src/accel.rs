use fugit::{ExtU64, TimerDurationU64, TimerInstantU64};
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagOneShot;
use lsm303agr::{Acceleration, Lsm303agr};

pub struct Accel<T: embedded_hal::i2c::I2c, const TIMER_HZ: u32> {
    sencer: Lsm303agr<I2cInterface<T>, MagOneShot>,
    max_diff_square: i32,
    last_accel_data: Option<Acceleration>,
    snake_event_fn: Option<fn()>,
    snake_sensitivity: i32,
    last_update_time: TimerInstantU64<TIMER_HZ>,
    debounce_ms: TimerDurationU64<TIMER_HZ>,
}

impl<T: embedded_hal::i2c::I2c, const TIMER_HZ: u32> Accel<T, TIMER_HZ> {
    pub fn new(sencer: Lsm303agr<I2cInterface<T>, MagOneShot>) -> Self {
        Self {
            sencer,
            max_diff_square: 0,
            last_accel_data: None,
            snake_event_fn: None,
            snake_sensitivity: 300_000,
            last_update_time: TimerInstantU64::from_ticks(0),
            debounce_ms: 400.millis(),
        }
    }

    pub fn attach_snake_event(&mut self, f: fn()) {
        self.snake_event_fn = Some(f);
    }

    pub fn set_snake_sensitivity(&mut self, snake_sensitivity: i32) {
        self.snake_sensitivity = snake_sensitivity;
    }

    pub fn tick(&mut self, now: &TimerInstantU64<TIMER_HZ>) {
        if self.update(now).is_some() {
            if self.max_diff_square > self.snake_sensitivity {
                self.reset();
                if let Some(f) = self.snake_event_fn {
                    f();
                }
            }
        }
    }

    fn update(&mut self, now: &TimerInstantU64<TIMER_HZ>) -> Option<Acceleration> {
        if *now - self.last_update_time > self.debounce_ms {
            self.last_update_time = *now;
            if self.sencer.accel_status().unwrap().xyz_new_data() {
                let new_accel_data = self.sencer.acceleration().unwrap();
                let last_accel_data = self.last_accel_data.unwrap_or(new_accel_data);
                let diff_square = self.diff_square(new_accel_data, last_accel_data);

                defmt::debug!("diff_square: {}", diff_square);

                self.max_diff_square = self.max_diff_square.max(diff_square);
                self.last_accel_data = Some(new_accel_data);
                return Some(new_accel_data);
            }
        }
        None
    }

    fn reset(&mut self) {
        self.max_diff_square = 0;
        self.last_accel_data = None;
    }

    fn diff_square(&self, a: Acceleration, b: Acceleration) -> i32 {
        let diff = (
            a.x_mg() - b.x_mg(),
            a.y_mg() - b.y_mg(),
            a.z_mg() - b.z_mg(),
        );
        diff.0.pow(2) + diff.1.pow(2) + diff.2.pow(2)
    }
}
