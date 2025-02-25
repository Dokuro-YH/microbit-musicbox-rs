use core::marker::PhantomData;

use embedded_hal::i2c::I2c;
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagOneShot;
use lsm303agr::{AccelMode, AccelOutputDataRate, Acceleration, Lsm303agr};
use microbit::hal::timer::{Instance as TimerInstance, Timer};

pub struct Accel<I, const TIMER_HZ: u32> {
    sensor: Lsm303agr<I2cInterface<I>, MagOneShot>,
    last_data: Option<Acceleration>,
}

impl<I, const TIMER_HZ: u32> Accel<I, TIMER_HZ>
where
    I: I2c,
{
    pub fn new_with_i2c(i2c: I) -> Self {
        let mut sensor = Lsm303agr::new_with_i2c(i2c);

        Self {
            sensor,
            last_data: None,
        }
    }

    pub fn init(&mut self) {
        self.sensor.init().unwrap();
    }

    pub fn tick(&mut self) {
        if self.sensor.accel_status().unwrap().xyz_new_data() {
            let data = self.sensor.acceleration().unwrap();
        }
        todo!()
    }
}
