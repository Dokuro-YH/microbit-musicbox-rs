use crate::melody::Melody;
use crate::tone::Tone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Playing { pos: usize, progress: usize },
    Paused { pos: usize, progress: usize },
    Stopped,
}

/// 命令执行后外壳需执行的硬件动作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEffect {
    Start,
    Stop,
    Restart,
    None,
}

/// 功能核心：播放状态机，无 I/O 依赖，可直接单测。
///
/// 维护曲目列表索引、音符推进、音量，外部通过 TIMER2 ISR 驱动音符推进。
pub struct PlayerModel<'a> {
    list: &'a [Melody],
    state: PlayerState,
    volume: u32,
}

impl<'a> PlayerModel<'a> {
    pub fn new(list: &'a [Melody]) -> Self {
        Self {
            list,
            state: PlayerState::Stopped,
            volume: 20,
        }
    }

    // ── 对外命令 ──

    pub fn play_or_resume(&mut self) -> PlayerEffect {
        match self.state {
            PlayerState::Stopped => {
                self.state = PlayerState::Playing {
                    pos: 0,
                    progress: 0,
                };
                PlayerEffect::Start
            }
            PlayerState::Paused { pos, progress } => {
                self.state = PlayerState::Playing { pos, progress };
                PlayerEffect::Start
            }
            PlayerState::Playing { .. } => PlayerEffect::None,
        }
    }

    pub fn pause(&mut self) -> PlayerEffect {
        match self.state {
            PlayerState::Playing { pos, progress } => {
                self.state = PlayerState::Paused { pos, progress };
                PlayerEffect::Stop
            }
            _ => PlayerEffect::None,
        }
    }

    pub fn next(&mut self) -> PlayerEffect {
        let next_pos = self.next_pos();
        self.state = PlayerState::Playing {
            pos: next_pos,
            progress: 0,
        };
        PlayerEffect::Restart
    }

    pub fn prev(&mut self) -> PlayerEffect {
        let prev_pos = self.prev_pos();
        self.state = PlayerState::Playing {
            pos: prev_pos,
            progress: 0,
        };
        PlayerEffect::Restart
    }

    pub fn set_list(&mut self, list: &'a [Melody]) -> PlayerEffect {
        self.list = list;
        self.state = PlayerState::Stopped;
        PlayerEffect::Stop
    }

    // ── 音量 ──

    pub fn volume_add(&mut self, vol: u32) {
        self.volume = self.volume.saturating_add(vol).min(100);
    }

    pub fn volume_sub(&mut self, vol: u32) {
        self.volume = self.volume.saturating_sub(vol);
    }

    pub fn volume(&self) -> u32 {
        self.volume
    }

    // ── 查询 ──

    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlayerState::Playing { .. })
    }

    // ── TIMER2 ISR 事件 ──

    /// play_fired: 返回当前应演奏的音符及持续时长。
    /// 若当前 progress 已超出旋律范围，自动回到 progress=0 取首个音符。
    pub fn on_play_fired(&mut self) -> Option<(Tone, u32)> {
        let (pos, progress) = match self.state {
            PlayerState::Playing { pos, progress } => (pos, progress),
            _ => return None,
        };

        if let Some(melody) = self.list.get(pos) {
            if let Some((tone, delay_ms)) = melody.get(progress) {
                return Some((tone, delay_ms));
            }
        }

        // 曲终回绕
        self.state = PlayerState::Playing { pos, progress: 0 };
        self.list.get(pos).and_then(|m| m.get(0))
    }

    /// next_fired: 推进到下一音符（停止当前音符）
    pub fn on_next_fired(&mut self) {
        if let PlayerState::Playing { pos, progress } = self.state {
            self.state = PlayerState::Playing {
                pos,
                progress: progress + 1,
            };
        }
    }

    // ── 内部 ──

    fn next_pos(&self) -> usize {
        let max = self.list.len().saturating_sub(1);
        let pos = self.current_pos();
        if pos >= max {
            0
        } else {
            pos + 1
        }
    }

    fn prev_pos(&self) -> usize {
        let max = self.list.len().saturating_sub(1);
        let pos = self.current_pos();
        if pos == 0 {
            max
        } else {
            pos - 1
        }
    }

    fn current_pos(&self) -> usize {
        match self.state {
            PlayerState::Playing { pos, .. } | PlayerState::Paused { pos, .. } => pos,
            PlayerState::Stopped => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_melody() -> Melody {
        Melody {
            whole_note_delay_ms: 1000,
            notes: &[(Tone::C4, 4), (Tone::D4, 4), (Tone::E4, 4)],
        }
    }

    fn empty_melody() -> Melody {
        Melody {
            whole_note_delay_ms: 1000,
            notes: &[],
        }
    }

    // ── 状态转换 ──

    #[test]
    fn initial_state_is_stopped() {
        let melodies = [test_melody()];
        let model = PlayerModel::new(&melodies);
        assert!(!model.is_playing());
    }

    #[test]
    fn stop_to_playing() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        let effect = model.play_or_resume();
        assert_eq!(effect, PlayerEffect::Start);
        assert!(model.is_playing());
    }

    #[test]
    fn play_or_resume_when_playing_is_noop() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();
        let effect = model.play_or_resume();
        assert_eq!(effect, PlayerEffect::None);
        assert!(model.is_playing());
    }

    #[test]
    fn pause_and_resume() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();

        let effect = model.pause();
        assert_eq!(effect, PlayerEffect::Stop);
        assert!(!model.is_playing());

        let effect = model.play_or_resume();
        assert_eq!(effect, PlayerEffect::Start);
        assert!(model.is_playing());
    }

    #[test]
    fn pause_when_stopped_is_noop() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        let effect = model.pause();
        assert_eq!(effect, PlayerEffect::None);
    }

    // ── 切歌 ──

    #[test]
    fn next_track() {
        let melodies = [test_melody(), empty_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();

        let effect = model.next();
        assert_eq!(effect, PlayerEffect::Restart);
    }

    #[test]
    fn next_wraps_to_first() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();
        model.next(); // 0→0 (only one)
    }

    #[test]
    fn prev_wraps_to_last() {
        let melodies = [test_melody(), empty_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();

        let effect = model.prev();
        assert_eq!(effect, PlayerEffect::Restart);
    }

    // ── 音符推进 ──

    #[test]
    fn on_play_fired_returns_first_note() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();

        let note = model.on_play_fired();
        assert_eq!(note, Some((Tone::C4, 250)));
    }

    #[test]
    fn on_next_fired_advances_progress() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();

        model.on_next_fired();
        let note = model.on_play_fired();
        assert_eq!(note, Some((Tone::D4, 250)));
    }

    #[test]
    fn melody_loops_at_end() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();

        for _ in 0..3 {
            model.on_next_fired();
        }
        let note = model.on_play_fired();
        assert_eq!(note, Some((Tone::C4, 250)));
    }

    #[test]
    fn on_play_fired_when_not_playing_returns_none() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        let note = model.on_play_fired();
        assert_eq!(note, None);
    }

    // ── 音量 ──

    #[test]
    fn volume_clamps_at_100() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.volume_add(200);
        assert_eq!(model.volume(), 100);
    }

    #[test]
    fn volume_clamps_at_0() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.volume_sub(50);
        assert_eq!(model.volume(), 0);
    }

    #[test]
    fn set_list_stops_playback() {
        let melodies = [test_melody()];
        let mut model = PlayerModel::new(&melodies);
        model.play_or_resume();
        assert!(model.is_playing());

        let effect = model.set_list(&[]);
        assert_eq!(effect, PlayerEffect::Stop);
        assert!(!model.is_playing());
    }
}
