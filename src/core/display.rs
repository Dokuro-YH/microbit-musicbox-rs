/// 显示事件：来自外部输入，驱动显示状态切换
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayEvent {
    /// 播放中
    Playing,
    /// 已暂停
    Paused,
    /// 已停止
    Stopped,

    /// 音量变更 (0..=100)
    Volume(u8),

    /// 下一曲
    NextTrack,
    /// 上一曲
    PreviousTrack,

    /// 摇动
    Shake,
}

/// 播放器状态（常驻背景层）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}

/// 覆盖层类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayKind {
    Volume(u8),
    NextTrack,
    PreviousTrack,
    Shake,
}

impl OverlayKind {
    fn timeout_ms(&self) -> u64 {
        match self {
            OverlayKind::Volume(_) => 800,
            _ => 500,
        }
    }
}

/// 覆盖层实例
#[derive(Debug, Clone, Copy)]
struct Overlay {
    kind: OverlayKind,
    start_ms: u64,
}

/// 显示图标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Playing,
    Paused,
    Stopped,
    Next,
    Prev,
    Shake,
}

/// 功能核心发出的渲染指令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayCommand {
    Icon(Icon),
    VolumeBar(u8),
}

/// 功能核心：显示屏状态机，无 I/O 依赖，可直接单测。
///
/// 双层显示模型：
/// - **常驻背景层**：PlayerStatus（Playing / Paused / Stopped）
/// - **临时覆盖层**：音量条 / 切歌箭头 / 摇动图标，超时后自动恢复背景
///
/// 播放状态切换会立即清除覆盖层；覆盖层事件会叠在背景之上，超时后恢复。
pub struct DisplayModel {
    player_status: PlayerStatus,
    overlay: Option<Overlay>,
}

impl DisplayModel {
    pub fn new() -> Self {
        Self {
            player_status: PlayerStatus::Stopped,
            overlay: None,
        }
    }

    /// 每帧调用。
    ///
    /// - `now_ms`: 当前时间戳 (ms)
    /// - `event`: 可选的外部事件
    ///
    /// 返回当前应显示的渲染指令。
    pub fn update(&mut self, now_ms: u64, event: Option<DisplayEvent>) -> DisplayCommand {
        if let Some(event) = event {
            self.handle_event(now_ms, event);
        }

        // 覆盖层超时 → 清除
        if let Some(ref overlay) = self.overlay {
            if now_ms.saturating_sub(overlay.start_ms) >= overlay.kind.timeout_ms() {
                self.overlay = None;
            }
        }

        // 优先渲染覆盖层，其次背景
        match &self.overlay {
            Some(overlay) => match overlay.kind {
                OverlayKind::Volume(vol) => DisplayCommand::VolumeBar(vol),
                OverlayKind::NextTrack => DisplayCommand::Icon(Icon::Next),
                OverlayKind::PreviousTrack => DisplayCommand::Icon(Icon::Prev),
                OverlayKind::Shake => DisplayCommand::Icon(Icon::Shake),
            },
            None => match self.player_status {
                PlayerStatus::Playing => DisplayCommand::Icon(Icon::Playing),
                PlayerStatus::Paused => DisplayCommand::Icon(Icon::Paused),
                PlayerStatus::Stopped => DisplayCommand::Icon(Icon::Stopped),
            },
        }
    }

    fn handle_event(&mut self, now_ms: u64, event: DisplayEvent) {
        use DisplayEvent::*;

        match event {
            Playing => {
                self.player_status = PlayerStatus::Playing;
                self.overlay = None;
            }
            Paused => {
                self.player_status = PlayerStatus::Paused;
                self.overlay = None;
            }
            Stopped => {
                self.player_status = PlayerStatus::Stopped;
                self.overlay = None;
            }
            Volume(vol) => {
                self.overlay = Some(Overlay {
                    kind: OverlayKind::Volume(vol),
                    start_ms: now_ms,
                });
            }
            NextTrack => {
                self.overlay = Some(Overlay {
                    kind: OverlayKind::NextTrack,
                    start_ms: now_ms,
                });
            }
            PreviousTrack => {
                self.overlay = Some(Overlay {
                    kind: OverlayKind::PreviousTrack,
                    start_ms: now_ms,
                });
            }
            Shake => {
                self.overlay = Some(Overlay {
                    kind: OverlayKind::Shake,
                    start_ms: now_ms,
                });
            }
        }
    }
}

impl Default for DisplayModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 背景层 ──

    #[test]
    fn initial_state_is_stopped() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, None);
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Stopped));
    }

    #[test]
    fn playing_event_shows_playing_icon() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, Some(DisplayEvent::Playing));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Playing));
    }

    #[test]
    fn paused_event_shows_paused_icon() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, Some(DisplayEvent::Paused));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Paused));
    }

    #[test]
    fn stopped_event_shows_stopped_icon() {
        let mut model = DisplayModel::new();
        model.update(0, Some(DisplayEvent::Playing));
        let cmd = model.update(100, Some(DisplayEvent::Stopped));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Stopped));
    }

    // ── 覆盖层 ──

    #[test]
    fn volume_overlay() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, Some(DisplayEvent::Volume(60)));
        assert_eq!(cmd, DisplayCommand::VolumeBar(60));
    }

    #[test]
    fn next_track_overlay() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, Some(DisplayEvent::NextTrack));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Next));
    }

    #[test]
    fn prev_track_overlay() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, Some(DisplayEvent::PreviousTrack));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Prev));
    }

    #[test]
    fn shake_overlay() {
        let mut model = DisplayModel::new();
        let cmd = model.update(0, Some(DisplayEvent::Shake));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Shake));
    }

    // ── 覆盖层超时 ──

    #[test]
    fn overlay_expires_and_returns_to_background() {
        let mut model = DisplayModel::new();
        model.update(0, Some(DisplayEvent::Playing));

        // 显示切歌覆盖层
        let cmd = model.update(100, Some(DisplayEvent::NextTrack));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Next));

        // 超时前仍显示覆盖层
        let cmd = model.update(400, None);
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Next));

        // 超时后恢复背景 (500ms)
        let cmd = model.update(700, None);
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Playing));
    }

    #[test]
    fn volume_overlay_expires_800ms() {
        let mut model = DisplayModel::new();
        model.update(0, Some(DisplayEvent::Playing));
        model.update(100, Some(DisplayEvent::Volume(80)));

        // 800ms 后超时
        let cmd = model.update(950, None);
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Playing));
    }

    // ── 播放状态切换清除覆盖层 ──

    #[test]
    fn playing_event_clears_overlay() {
        let mut model = DisplayModel::new();
        model.update(0, Some(DisplayEvent::Volume(50)));
        // 播放状态切换立即清除覆盖层
        let cmd = model.update(100, Some(DisplayEvent::Playing));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Playing));
    }

    #[test]
    fn paused_event_clears_overlay() {
        let mut model = DisplayModel::new();
        model.update(0, Some(DisplayEvent::Volume(50)));
        let cmd = model.update(100, Some(DisplayEvent::Paused));
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Paused));
    }

    #[test]
    fn event_updates_then_expires_in_same_tick() {
        let mut model = DisplayModel::new();
        // 同时收到事件和时间推进
        // 事件先处理，然后再检查超时
        let cmd = model.update(600, Some(DisplayEvent::NextTrack));
        // 事件刚设置，600ms > 500ms timeout，但 start_ms = 600，所以未超时
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Next));

        // 601ms 时仍未超时
        let cmd = model.update(1100, None);
        // 1100 - 600 = 500 >= 500 → 超时
        // 实际是 >= 500 超时
        assert_eq!(cmd, DisplayCommand::Icon(Icon::Stopped));
    }
}
