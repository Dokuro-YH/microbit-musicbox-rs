use fugit::TimerInstantU64;
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagOneShot;
use lsm303agr::{Acceleration, Lsm303agr};

use musicbox::shake_detector::ShakeDetector;

/// Imperative Shell：负责传感器 I/O 读取、时间转换和回调调度。
/// 摇动检测的核心状态机委托给功能核心 [`ShakeDetector`]。
pub struct Accel<T: embedded_hal::i2c::I2c, const TIMER_HZ: u32> {
    /// 加速度传感器实例
    sensor: Lsm303agr<I2cInterface<T>, MagOneShot>,
    /// 功能核心：纯摇动检测状态机
    detector: ShakeDetector,
    /// 摇动事件回调
    shake_event_fn: Option<fn()>,
}

impl<T: embedded_hal::i2c::I2c, const TIMER_HZ: u32> Accel<T, TIMER_HZ> {
    pub fn new(sensor: Lsm303agr<I2cInterface<T>, MagOneShot>) -> Self {
        Self {
            sensor,
            detector: ShakeDetector::new(),
            shake_event_fn: None,
        }
    }

    /// 注册摇动事件回调
    pub fn attach_shake_event(&mut self, f: fn()) {
        self.shake_event_fn = Some(f);
    }

    /// 每帧 tick：读取传感器 → 喂入功能核心 → 触发回调
    pub fn tick(&mut self, now: &TimerInstantU64<TIMER_HZ>) {
        let Some(accel) = self.accel_new_data() else {
            return;
        };

        // 将 timer ticks 转换为毫秒
        let now_ms = now.duration_since_epoch().ticks() / (TIMER_HZ as u64 / 1000);

        if self
            .detector
            .update(now_ms, accel.x_mg(), accel.y_mg(), accel.z_mg())
            .is_some()
        {
            if let Some(callback) = self.shake_event_fn {
                callback();
            }
        }
    }

    /// 尝试从传感器读取一帧新数据
    fn accel_new_data(&mut self) -> Option<Acceleration> {
        if self.sensor.accel_status().unwrap().xyz_new_data() {
            Some(self.sensor.acceleration().unwrap())
        } else {
            None
        }
    }
}
