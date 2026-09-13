/// 按钮事件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    /// 单击
    Click,
    /// 双击
    DoubleClick,
    /// 多次点击（>= 3 次）
    MultiClick(u32),
    /// 长按开始
    LongPressStart,
    /// 长按持续中（每 tick 触发）
    LongPressDuring,
    /// 长按结束
    LongPressStop,
}

/// 内部状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    Down,
    Up,
    Count,
    Press,
    Pressend,
}

/// 功能核心：按钮检测状态机，无 I/O 依赖，可直接单测。
///
/// 检测逻辑：
/// 1. 按下 → 进入 Down，记录按下时间
/// 2. 释放且超过消抖 → 进入 Up，等待再次按下或超时
/// 3. 连续按下/释放累计点击次数
/// 4. 点击超时 → 触发 Click / DoubleClick / MultiClick
/// 5. 按下超长按阈值 → 触发 LongPressStart，随后每 tick LongPressDuring
/// 6. 长按释放 → 触发 LongPressStop
pub struct ButtonDetector {
    state: State,
    click_count: u32,
    /// 进入当前状态的时间 (ms)
    transition_time_ms: u64,

    /// 消抖时间 (ms)
    pub debounce_ms: u64,
    /// 点击超时 (ms)，超过此时间未再次按下则判定点击结束
    pub click_ms: u64,
    /// 长按阈值 (ms)，按下超过此时间触发长按
    pub press_ms: u64,
}

impl ButtonDetector {
    /// 使用默认参数创建：
    /// - debounce_ms = 50
    /// - click_ms = 200
    /// - press_ms = 500
    pub fn new() -> Self {
        Self {
            state: State::Pending,
            click_count: 0,
            transition_time_ms: 0,

            debounce_ms: 50,
            click_ms: 200,
            press_ms: 500,
        }
    }

    /// 每帧喂入按钮状态，若触发事件则返回 `Some(ButtonEvent)`。
    ///
    /// - `now_ms`: 当前时间戳 (ms)，单调递增
    /// - `active`: 按钮是否处于按下状态（true = 按下）
    pub fn update(&mut self, now_ms: u64, active: bool) -> Option<ButtonEvent> {
        use State::*;

        let wait_time = now_ms.saturating_sub(self.transition_time_ms);

        match self.state {
            Pending => {
                if active {
                    self.transition_to(Down);
                    self.click_count = 0;
                    self.transition_time_ms = now_ms;
                }
                None
            }
            Down => {
                if !active && wait_time > self.debounce_ms {
                    self.transition_to(Up);
                    None
                } else if active && wait_time > self.press_ms {
                    self.transition_to(Press);
                    Some(ButtonEvent::LongPressStart)
                } else {
                    None
                }
            }
            Up => {
                if !active && wait_time > self.debounce_ms {
                    self.click_count += 1;
                    self.transition_to(Count);
                }
                None
            }
            Count => {
                if active {
                    self.transition_to(Down);
                    self.transition_time_ms = now_ms;
                    None
                } else if wait_time > self.click_ms {
                    let event = match self.click_count {
                        1 => ButtonEvent::Click,
                        2 => ButtonEvent::DoubleClick,
                        cnt => ButtonEvent::MultiClick(cnt),
                    };
                    self.reset();
                    Some(event)
                } else {
                    None
                }
            }
            Press => {
                if !active {
                    self.transition_to(Pressend);
                    self.transition_time_ms = now_ms;
                    None
                } else {
                    Some(ButtonEvent::LongPressDuring)
                }
            }
            Pressend => {
                if !active && wait_time > self.debounce_ms {
                    self.reset();
                    Some(ButtonEvent::LongPressStop)
                } else {
                    None
                }
            }
        }
    }

    fn transition_to(&mut self, state: State) {
        self.state = state;
    }

    fn reset(&mut self) {
        self.state = State::Pending;
        self.click_count = 0;
        self.transition_time_ms = 0;
    }
}

impl Default for ButtonDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 单击 ──

    #[test]
    fn single_click() {
        let mut det = ButtonDetector::new();
        // 初始无事件
        assert_eq!(det.update(0, false), None);

        // 按下
        assert_eq!(det.update(10, true), None);
        // 释放（消抖后）
        assert_eq!(det.update(70, false), None);
        // 消抖完成 → Count
        assert_eq!(det.update(130, false), None);
        // 点击超时 → Click
        assert_eq!(det.update(340, false), Some(ButtonEvent::Click));
    }

    // ── 双击 ──

    #[test]
    fn double_click() {
        let mut det = ButtonDetector::new();
        // 第一次点击
        det.update(0, true); // 按下
        det.update(60, false); // 释放 → Up
        det.update(120, false); // → Count

        // 在 click_ms(200) 内再次按下
        assert_eq!(det.update(150, true), None); // 回到 Down，click_count 保持
        det.update(210, false); // 释放 → Up
        det.update(270, false); // → Count，click_count=2
                                // 超时 → DoubleClick
        assert_eq!(det.update(480, false), Some(ButtonEvent::DoubleClick));
    }

    // ── 三击 ──

    #[test]
    fn triple_click() {
        let mut det = ButtonDetector::new();
        // 三次快速点击
        for i in 0..3 {
            let base = i * 150;
            det.update(base, true);
            det.update(base + 60, false);
            det.update(base + 120, false);
            if i < 2 {
                det.update(base + 140, true); // 下次按下
            }
        }
        // 超时 → MultiClick(3)
        assert_eq!(det.update(600, false), Some(ButtonEvent::MultiClick(3)));
    }

    // ── 长按 ──

    #[test]
    fn long_press_start_and_during() {
        let mut det = ButtonDetector::new();
        det.update(0, true); // 按下

        // 长按阈值 500ms
        assert_eq!(det.update(510, true), Some(ButtonEvent::LongPressStart));
        // 持续触发 LongPressDuring
        assert_eq!(det.update(520, true), Some(ButtonEvent::LongPressDuring));
        assert_eq!(det.update(530, true), Some(ButtonEvent::LongPressDuring));
    }

    #[test]
    fn long_press_release() {
        let mut det = ButtonDetector::new();
        det.update(0, true);
        det.update(510, true); // LongPressStart
        assert_eq!(det.update(520, false), None); // 释放 → Pressend
                                                  // 消抖后 → LongPressStop
        assert_eq!(det.update(580, false), Some(ButtonEvent::LongPressStop));
        // 之后回到 Pending
        assert_eq!(det.update(590, false), None);
        assert_eq!(det.update(590, true), None); // 可以再次按下
    }

    // ── 消抖 ──

    #[test]
    fn debounce_ignores_short_pulses() {
        let mut det = ButtonDetector::new();
        det.update(0, true); // 按下
                             // 10ms 后释放，未达到 debounce_ms(50)
        assert_eq!(det.update(10, false), None);
        // 仍在 Down 状态
        assert_eq!(det.update(20, false), None);
        // 再次按下也在 Down
        assert_eq!(det.update(30, true), None);
        // 长按仍可触发
        assert_eq!(det.update(540, true), Some(ButtonEvent::LongPressStart));
    }

    // ── 长按中意外释放再恢复 ──

    #[test]
    fn long_press_release_then_press_again_ignored() {
        let mut det = ButtonDetector::new();
        det.update(0, true);
        det.update(510, true); // LongPressStart

        // 释放 → Pressend
        det.update(520, false);
        // 消抖前又按下 → 仍为 Pressend，不触发事件
        assert_eq!(det.update(530, true), None);
        // 最终释放消抖后 → LongPressStop
        assert_eq!(det.update(590, false), Some(ButtonEvent::LongPressStop));
    }
}
