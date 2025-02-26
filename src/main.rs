#![no_std]
#![no_main]

extern crate microbit as bsp; // board support package

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use rtic_monotonics::nrf::timer::prelude::*;
nrf_timer0_monotonic!(Mono, 1_000_000);

// mod accel;
mod button;
mod melody;
mod player;
mod tone;

#[rtic::app(device = bsp::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use bsp::hal::clocks::Clocks;
    use bsp::hal::delay::Delay;
    use bsp::hal::gpio::{Input, Pin, PullUp};
    use bsp::hal::rtc::{Rtc, RtcInterrupt};
    use bsp::hal::twim;
    use bsp::pac::twim0::frequency::FREQUENCY_A;
    use bsp::pac::{PWM1, RTC0, TIMER1, TIMER2, TWIM0};
    use bsp::Board;

    use lsm303agr::interface::I2cInterface;
    use lsm303agr::mode::MagOneShot;
    use lsm303agr::{AccelMode, AccelOutputDataRate, Acceleration, Lsm303agr};

    type Accel = Lsm303agr<I2cInterface<twim::Twim<TWIM0>>, MagOneShot>;
    type Button = button::Button<Pin<Input<PullUp>>, 100>;
    type Display = bsp::display::nonblocking::Display<TIMER1>;
    type Player = player::Player<'static, TIMER2, PWM1>;

    const MELODY_LIST: &[melody::Melody] = &[
        melody::SUPER_MARIOBROS,
        melody::GAME_OF_THRONES,
        melody::MERRY_CHRISTMAS,
        melody::HAPPY_BIRTHDAY,
    ];

    #[shared]
    struct Shared {
        accel: Accel,
        btn1: Button,
        btn2: Button,
        display: Display,
        player: Player,
    }

    #[local]
    struct Local {
        rtc0: Rtc<RTC0>,
        accel_data: Option<Acceleration>,
        max_diff_square: i32,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        defmt::info!("init musicbox");

        // Create board
        let board = Board::new(ctx.device, ctx.core);

        // Initialize Monotonic
        Mono::start(board.TIMER0);

        // Starting the low-frequency clock (needed for RTC to work)
        Clocks::new(board.CLOCK).start_lfclk();

        // RTC at 100Hz (32_768 / (327 + 1))
        // 100Hz; 10ms period
        let mut rtc0 = Rtc::new(board.RTC0, 327).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        // Accel
        let i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
        let accel = {
            let mut accel = Lsm303agr::new_with_i2c(i2c);

            let mut delay = Delay::new(board.SYST);
            accel.init().unwrap();
            accel
                .set_accel_mode_and_odr(&mut delay, AccelMode::Normal, AccelOutputDataRate::Hz10)
                .unwrap();
            accel
        };

        // Button A
        let btn1 = {
            let pin = board.buttons.button_a.into_pullup_input().degrade();
            let mut btn = Button::new(pin);
            btn.attach_event(|event| {
                handle_btn1_event::spawn(event).ok();
            });
            btn
        };

        // Button B
        let btn2 = {
            let pin = board.buttons.button_b.into_pullup_input().degrade();
            let mut btn = Button::new(pin);
            btn.attach_event(|event| {
                handle_btn2_event::spawn(event).ok();
            });
            btn
        };

        // Display
        let display = {
            let pins = board.display_pins;
            Display::new(board.TIMER1, pins)
        };

        // Player
        let player = {
            let pin = board
                .speaker_pin
                .into_push_pull_output(bsp::hal::gpio::Level::High)
                .degrade();
            Player::new(board.TIMER2, board.PWM1, pin, MELODY_LIST)
        };

        play_and_pause::spawn().ok();

        (
            Shared {
                accel,
                btn1,
                btn2,
                display,
                player,
            },
            Local {
                rtc0,
                accel_data: None,
                max_diff_square: 0,
            },
        )
    }

    #[task(binds = RTC0, local = [rtc0], shared = [btn1, btn2])]
    fn rtc0(mut ctx: rtc0::Context) {
        ctx.local.rtc0.reset_event(RtcInterrupt::Tick);
        ctx.shared.btn1.lock(|btn| btn.tick());
        ctx.shared.btn2.lock(|btn| btn.tick());
    }

    #[task(priority = 1, shared = [player])]
    async fn handle_btn1_event(mut ctx: handle_btn1_event::Context, event: button::Event) {
        use button::Event::*;

        defmt::debug!("btn1 event: {:?}", &event);
        ctx.shared.player.lock(|ply| match event {
            Click => {
                defmt::info!("volume - 10");
                ply.volume_sub(10);
            }
            LongPressStart | LongPressDuring | LongPressStop => {
                defmt::info!("volume - 1");
                ply.volume_sub(1);
            }
            DoubleClick => {
                defmt::info!("prev music");
                ply.prev();
            }
            _ => {}
        })
    }

    #[task(priority = 1, shared = [player])]
    async fn handle_btn2_event(mut ctx: handle_btn2_event::Context, event: button::Event) {
        use button::Event::*;

        defmt::debug!("btn2 event: {:?}", &event);
        ctx.shared.player.lock(|ply| match event {
            Click => {
                defmt::info!("volume + 10");
                ply.volume_add(10);
            }
            LongPressStart | LongPressDuring | LongPressStop => {
                defmt::info!("volume + 1");
                ply.volume_add(1);
            }
            DoubleClick => {
                defmt::info!("next music");
                ply.next();
            }
            _ => {}
        })
    }

    #[task(binds = TIMER1, shared = [display])]
    fn handle_display_event(mut ctx: handle_display_event::Context) {
        ctx.shared
            .display
            .lock(|display| display.handle_display_event());
    }

    #[task(binds = TIMER2, shared = [player])]
    fn handle_player_event(mut ctx: handle_player_event::Context) {
        ctx.shared.player.lock(|ply| ply.handle_play_event());
    }

    #[task(priority = 1, local = [accel_data, max_diff_square], shared = [accel, player])]
    async fn play_and_pause(mut ctx: play_and_pause::Context) {
        let format_accel =
            |a: Option<Acceleration>| a.map_or((0, 0, 0), |v| (v.x_mg(), v.y_mg(), v.z_mg()));

        loop {
            let old_data = *ctx.local.accel_data;
            let new_data = ctx.shared.accel.lock(|accel| {
                accel
                    .accel_status()
                    .unwrap()
                    .xyz_new_data()
                    .then(|| accel.acceleration().unwrap())
            });

            defmt::debug!(
                "Acceleration, old_data={}, new_data={}",
                format_accel(old_data),
                format_accel(new_data),
            );

            match (new_data, old_data) {
                (Some(new), Some(old)) => {
                    // 使用元组计算差值更紧凑
                    let diff = (
                        new.x_mg() - old.x_mg(),
                        new.y_mg() - old.y_mg(),
                        new.z_mg() - old.z_mg(),
                    );
                    let max_diff_square = *ctx.local.max_diff_square;
                    let diff_square = diff.0.pow(2) + diff.1.pow(2) + diff.2.pow(2);

                    *ctx.local.max_diff_square = max_diff_square.max(diff_square);

                    defmt::debug!("max_diff_square={}", ctx.local.max_diff_square);

                    if *ctx.local.max_diff_square > 300_000 {
                        ctx.shared.player.lock(|player| {
                            if player.is_playing() {
                                defmt::info!("pause");
                                player.pause();
                            } else {
                                defmt::info!("play");
                                player.play();
                            }
                        });
                        // 触发后重置状态
                        *ctx.local.accel_data = None;
                        *ctx.local.max_diff_square = 0;
                    } else {
                        *ctx.local.accel_data = Some(new);
                    }
                }
                (Some(new), None) => {
                    *ctx.local.accel_data = Some(new);
                }
                _ => {}
            }

            Mono::delay(400.millis()).await;
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }
}
