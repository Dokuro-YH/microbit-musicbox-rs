use microbit::{
    display::nonblocking::{Display as HwDisplay, Frame, GreyscaleImage, MicrobitFrame},
    gpio::DisplayPins,
    hal::timer,
};

use musicbox::display_state::{DisplayCommand, DisplayEvent, DisplayModel, Icon};

/// LED 亮度
const BRIGHT: u8 = 7;

/// Imperative Shell：封装硬件 Display，负责帧缓冲管理与 LED 矩阵渲染。
/// 显示状态机委托给功能核心 [`DisplayModel`]。
pub struct Display<T: timer::Instance> {
    hw: HwDisplay<T>,
    model: DisplayModel,
    frame: MicrobitFrame,
}

impl<T: timer::Instance> Display<T> {
    pub fn new(timer: T, pins: DisplayPins) -> Self {
        let mut frame = MicrobitFrame::default();
        // 清屏
        frame.set(&GreyscaleImage::new(&[[0; 5]; 5]));

        Self {
            hw: HwDisplay::new(timer, pins),
            model: DisplayModel::new(),
            frame,
        }
    }

    /// 硬件刷新回调：在 TIMER ISR 中调用
    pub fn handle_display_event(&mut self) {
        self.hw.handle_display_event();
    }

    /// 更新显示内容。
    ///
    /// - `now_ms`: 当前时间戳 (ms)
    /// - `event`: 可选的外部事件（None 时仅处理覆盖层超时）
    pub fn update(&mut self, now_ms: u64, event: Option<DisplayEvent>) {
        let cmd = self.model.update(now_ms, event);
        self.render(cmd);
    }

    fn render(&mut self, cmd: DisplayCommand) {
        let image = render_command(cmd);
        self.frame.set(&image);
        self.hw.show_frame(&self.frame);
    }
}

/// 将 DisplayCommand 渲染为 GreyscaleImage
fn render_command(cmd: DisplayCommand) -> GreyscaleImage {
    match cmd {
        DisplayCommand::Icon(icon) => icon_image(icon),
        DisplayCommand::VolumeBar(vol) => volume_bar_image(vol),
    }
}

fn icon_image(icon: Icon) -> GreyscaleImage {
    let pat = match icon {
        Icon::Playing => PLAYING,
        Icon::Paused => PAUSED,
        Icon::Stopped => STOPPED,
        Icon::Next => NEXT,
        Icon::Prev => PREV,
        Icon::Shake => SHAKE,
    };
    GreyscaleImage::new(&pat)
}

fn volume_bar_image(vol: u8) -> GreyscaleImage {
    let cols = (vol.clamp(0, 100) / 20).min(5) as usize;
    let mut pat = [[0u8; 5]; 5];
    for row in pat.iter_mut() {
        for (c, cell) in row.iter_mut().enumerate() {
            if c < cols {
                *cell = BRIGHT;
            }
        }
    }
    GreyscaleImage::new(&pat)
}

// ── 5×5 LED 矩阵图案 ──

/// ▶ Playing
const PLAYING: [[u8; 5]; 5] = [
    [0, 0, BRIGHT, 0, 0],
    [0, 0, BRIGHT, BRIGHT, 0],
    [0, 0, BRIGHT, BRIGHT, BRIGHT],
    [0, 0, BRIGHT, BRIGHT, 0],
    [0, 0, BRIGHT, 0, 0],
];

/// ⏸ Paused
const PAUSED: [[u8; 5]; 5] = [
    [0, BRIGHT, 0, BRIGHT, 0],
    [0, BRIGHT, 0, BRIGHT, 0],
    [0, BRIGHT, 0, BRIGHT, 0],
    [0, BRIGHT, 0, BRIGHT, 0],
    [0, BRIGHT, 0, BRIGHT, 0],
];

/// ■ Stopped
const STOPPED: [[u8; 5]; 5] = [
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
];

/// → Next
const NEXT: [[u8; 5]; 5] = [
    [0, 0, BRIGHT, 0, 0],
    [0, 0, 0, BRIGHT, 0],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [0, 0, 0, BRIGHT, 0],
    [0, 0, BRIGHT, 0, 0],
];

/// ← Prev
const PREV: [[u8; 5]; 5] = [
    [0, 0, BRIGHT, 0, 0],
    [0, BRIGHT, 0, 0, 0],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [0, BRIGHT, 0, 0, 0],
    [0, 0, BRIGHT, 0, 0],
];

/// ✦ Shake
const SHAKE: [[u8; 5]; 5] = [
    [0, 0, BRIGHT, 0, 0],
    [0, BRIGHT, BRIGHT, BRIGHT, 0],
    [BRIGHT, BRIGHT, BRIGHT, BRIGHT, BRIGHT],
    [0, BRIGHT, BRIGHT, BRIGHT, 0],
    [0, 0, BRIGHT, 0, 0],
];
