use fugit::{ExtU64, TimerDurationU64, TimerInstantU64};
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagOneShot;
use lsm303agr::{Acceleration, Lsm303agr};

// 状态机枚举
#[derive(Debug, Clone, Copy, PartialEq)]
enum ShakeState {
    Idle,
    Detecting,
    Debounce,
}

pub struct Accel<T: embedded_hal::i2c::I2c, const TIMER_HZ: u32> {
    // 传感器实例
    sensor: Lsm303agr<I2cInterface<T>, MagOneShot>,
    // 状态机状态
    state: ShakeState,
    // 上一次加速度值
    last_accel: Option<Acceleration>,
    // 计时器起点
    timer_start: Option<TimerInstantU64<TIMER_HZ>>,
    // 摇动计数
    hit_count: u32,
    // 回调函数
    shake_event_fn: Option<fn()>,
    // 参数：加速度变化平方阈值
    threshold_squared: i32,
    // 参数：检测时间窗口
    time_window: TimerDurationU64<TIMER_HZ>,
    // 参数：触发所需次数
    hit_count_threshold: u32,
    // 参数：去抖时间
    debounce_duration: TimerDurationU64<TIMER_HZ>,
}

impl<T: embedded_hal::i2c::I2c, const TIMER_HZ: u32> Accel<T, TIMER_HZ> {
    pub fn new(sensor: Lsm303agr<I2cInterface<T>, MagOneShot>) -> Self {
        Self {
            sensor,
            state: ShakeState::Idle,
            last_accel: None,
            timer_start: None,
            hit_count: 0,
            shake_event_fn: None,
            threshold_squared: (1500_i32).pow(2),
            time_window: 300.millis(),
            hit_count_threshold: 2,
            debounce_duration: 200.millis(),
        }
    }

    pub fn attach_shake_event(&mut self, f: fn()) {
        self.shake_event_fn = Some(f);
    }

    pub fn tick(&mut self, now: &TimerInstantU64<TIMER_HZ>) {
        let Some(accel) = self.accel_new_data() else {
            return;
        };

        if let Some(last_accel) = self.last_accel {
            let delta_sq = self.diff_square(accel, last_accel);

            match self.state {
                ShakeState::Idle => {
                    if delta_sq >= self.threshold_squared {
                        self.state = ShakeState::Detecting;
                        self.timer_start = Some(*now);
                        self.hit_count = 1;
                    }
                }

                ShakeState::Detecting => {
                    if let Some(start) = self.timer_start {
                        let elapsed = now.checked_duration_since(start).unwrap();

                        // 在时间窗口内
                        if elapsed <= self.time_window {
                            if delta_sq >= self.threshold_squared {
                                self.hit_count += 1;
                            }
                        } else {
                            // 时间窗口结束，检查是否触发
                            if self.hit_count >= self.hit_count_threshold {
                                if let Some(callback) = self.shake_event_fn {
                                    callback();
                                }
                                self.state = ShakeState::Debounce;
                                self.timer_start = Some(*now);
                            } else {
                                self.state = ShakeState::Idle;
                            }
                            self.hit_count = 0;
                        }
                    }
                }

                ShakeState::Debounce => {
                    if let Some(start) = self.timer_start {
                        let elapsed = now.checked_duration_since(start).unwrap();
                        if elapsed >= self.debounce_duration {
                            self.state = ShakeState::Idle;
                        }
                    }
                }
            }
        }

        // 更新上一次加速度值
        self.last_accel = Some(accel);
    }

    // 获取加速度数据
    fn accel_new_data(&mut self) -> Option<Acceleration> {
        if self.sensor.accel_status().unwrap().xyz_new_data() {
            Some(self.sensor.acceleration().unwrap())
        } else {
            None
        }
    }

    // 计算三轴差分平方和
    fn diff_square(&self, a: Acceleration, b: Acceleration) -> i32 {
        let dx = a.x_mg() - b.x_mg();
        let dy = a.y_mg() - b.y_mg();
        let dz = a.z_mg() - b.z_mg();
        dx.pow(2) + dy.pow(2) + dz.pow(2)
    }
}
