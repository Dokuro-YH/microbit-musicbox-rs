use bsp::hal::{
    gpio::{Output, Pin, PushPull},
    pwm, timer,
};
use fugit::ExtU32;

use musicbox::{
    melody::Melody,
    player_core::{PlayerEffect, PlayerModel},
    tone::Tone,
};

use self::inner::{PlayerBuzzer, PlayerTimer};

/// Imperative Shell：封装蜂鸣器 PWM 和硬件定时器。
/// 播放状态机委托给功能核心 [`PlayerModel`]。
pub struct Player<'a, T: timer::Instance, P: pwm::Instance> {
    model: PlayerModel<'a>,
    timer: PlayerTimer<T>,
    buzzer: PlayerBuzzer<P>,
}

impl<'a, T: timer::Instance, P: pwm::Instance> Player<'a, T, P> {
    pub fn new(timer: T, pwm: P, pin: Pin<Output<PushPull>>, list: &'a [Melody]) -> Self {
        Self {
            model: PlayerModel::new(list),
            timer: PlayerTimer::new(timer),
            buzzer: PlayerBuzzer::new(pwm, pin),
        }
    }

    // ── 查询 ──

    pub fn is_playing(&self) -> bool {
        self.model.is_playing()
    }

    pub fn volume(&self) -> u32 {
        self.model.volume()
    }

    // ── 对外命令 → 委托 model → apply ──

    pub fn volume_add(&mut self, volume: u32) {
        self.model.volume_add(volume);
    }

    pub fn volume_sub(&mut self, volume: u32) {
        self.model.volume_sub(volume);
    }

    pub fn set_list(&mut self, list: &'a [Melody]) {
        let effect = self.model.set_list(list);
        self.apply(effect);
    }

    pub fn play_or_resume(&mut self) {
        let effect = self.model.play_or_resume();
        self.apply(effect);
    }

    pub fn pause(&mut self) {
        let effect = self.model.pause();
        self.apply(effect);
    }

    pub fn next(&mut self) {
        let effect = self.model.next();
        self.apply(effect);
    }

    pub fn prev(&mut self) {
        let effect = self.model.prev();
        self.apply(effect);
    }

    // ── TIMER2 ISR ──

    pub fn handle_play_event(&mut self) {
        let play_fired = self.timer.check_play();
        let next_fired = self.timer.check_next();

        if play_fired {
            if let Some((tone, delay_ms)) = self.model.on_play_fired() {
                self.buzzer.tone(tone, self.model.volume());
                self.timer.set_play_duration((delay_ms * 1_000).micros());
                self.timer.set_next_duration((delay_ms * 900).micros());
            } else {
                // 不应出现（Playing 状态总应有音符）
                self.buzzer.stop();
                self.timer.stop();
            }
        }

        if next_fired {
            self.model.on_next_fired();
            self.buzzer.stop();
        }
    }

    // ── 硬件效应汇聚点 ──

    fn apply(&mut self, effect: PlayerEffect) {
        use PlayerEffect::*;
        match effect {
            Start => {
                self.timer.start();
                self.timer.set_play_duration(1.secs());
            }
            Stop => {
                self.buzzer.stop();
                self.timer.stop();
            }
            Restart => {
                self.buzzer.stop();
                self.timer.stop();
                self.timer.start();
                self.timer.set_play_duration(1.secs());
            }
            None => {}
        }
    }
}

mod inner {
    use super::*;
    use fugit::{TimerDurationU32, TimerInstantU32};

    type Instant = TimerInstantU32<1_000_000>;
    type Duration = TimerDurationU32<1_000_000>;

    pub(super) struct PlayerBuzzer<T: pwm::Instance>(pwm::Pwm<T>);

    impl<T: pwm::Instance> PlayerBuzzer<T> {
        pub fn new(pwm: T, pin: Pin<Output<PushPull>>) -> Self {
            let buzzer = pwm::Pwm::new(pwm);
            buzzer
                .set_counter_mode(pwm::CounterMode::UpAndDown)
                .set_output_pin(pwm::Channel::C0, pin)
                .disable();
            Self(buzzer)
        }

        pub fn tone(&self, tone: Tone, volume: u32) {
            if tone == Tone::REST {
                self.0.disable();
            } else {
                self.0.disable();
                self.0.set_period(tone.hz());
                self.set_volume(volume);
                self.0.enable();
            }
        }

        pub fn stop(&self) {
            self.0.disable();
        }

        #[inline(always)]
        fn set_volume(&self, volume: u32) {
            let volume = volume.clamp(0, 100) as f32;
            let max_duty = self.0.max_duty() as f32;
            let max_vol = max_duty * 0.5;
            let min_vol = max_duty * 0.2;
            let vol_range = max_vol - min_vol;
            let target_duty = min_vol + (vol_range * (volume / 100.0));
            self.0.set_duty_on(pwm::Channel::C0, target_duty as u16);
        }
    }

    pub(super) struct PlayerTimer<T: timer::Instance>(T);

    impl<T: timer::Instance> PlayerTimer<T> {
        pub fn new(timer: T) -> Self {
            let timer0 = timer.as_timer0();
            timer0.tasks_stop.write(|w| w.tasks_stop().set_bit());
            timer0.tasks_clear.write(|w| w.tasks_clear().set_bit());
            timer0.bitmode.write(|w| w.bitmode()._32bit());
            timer0.prescaler.write(|w| unsafe { w.prescaler().bits(4) }); // 1 Mhz
            timer0.intenset.write(|w| w.compare1().set_bit());
            timer0.intenset.write(|w| w.compare2().set_bit());
            Self(timer)
        }

        pub fn start(&self) {
            let timer = self.0.as_timer0();
            timer.tasks_start.write(|w| unsafe { w.bits(1) });
        }

        pub fn stop(&self) {
            let timer = self.0.as_timer0();
            timer.tasks_stop.write(|w| unsafe { w.bits(1) });
            timer.tasks_clear.write(|w| unsafe { w.bits(1) });
        }

        pub fn set_play_duration(&self, duration: Duration) {
            self.set_duration_for_cc(1, duration)
        }

        pub fn set_next_duration(&self, duration: Duration) {
            self.set_duration_for_cc(2, duration)
        }

        pub fn check_play(&self) -> bool {
            self.check_fired_for_cc(1)
        }

        pub fn check_next(&self) -> bool {
            self.check_fired_for_cc(2)
        }

        #[inline(always)]
        pub fn now(&self) -> Instant {
            let timer = self.0.as_timer0();
            timer.tasks_capture[0].write(|w| unsafe { w.bits(1) });
            Instant::from_ticks(timer.cc[0].read().bits())
        }

        #[inline(always)]
        fn set_duration_for_cc(&self, pos: usize, duration: Duration) {
            let timer = self.0.as_timer0();
            let now = self.now();
            let instant = now + duration;
            timer.cc[pos].write(|w| unsafe { w.cc().bits(instant.duration_since_epoch().ticks()) });
        }

        #[inline(always)]
        fn check_fired_for_cc(&self, pos: usize) -> bool {
            let timer = self.0.as_timer0();
            let reg = &timer.events_compare[pos];
            let fired = reg.read().bits() != 0;
            if fired {
                reg.reset();
            }
            fired
        }
    }
}
