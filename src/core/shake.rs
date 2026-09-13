/// 摇动检测事件
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShakeEvent {
    /// 检测到一次有效摇动
    Shake,
}

/// 功能核心：纯状态机，无 I/O 依赖，可直接单测。
///
/// 检测逻辑：
/// 1. 逐帧计算加速度变化量（三轴平方和），与 `threshold_squared` 比较
/// 2. 首次剧烈运动（超过阈值）开启一个 `window_ms` 的检测窗口
/// 3. 窗口内累计 `required_hits` 次剧烈运动即触发 `ShakeEvent`
/// 4. 触发后进入 `cooldown_ms` 冷却期，期间忽略所有输入
/// 5. 窗口超时未达到命中次数则重置
pub struct ShakeDetector {
    /// 上一帧加速度值 (x, y, z) in mg
    last: Option<(i32, i32, i32)>,

    /// 当前窗口内命中次数
    hit_count: u8,
    /// 当前窗口起始时间 (ms)
    window_start_ms: u64,
    /// 冷却期结束时间 (ms)
    cooldown_until_ms: u64,

    /// 加速度变化平方阈值（约 1200mg 的帧间变化）
    threshold_squared: i64,
    /// 窗口内触发所需命中次数
    required_hits: u8,
    /// 检测窗口时长 (ms)
    window_ms: u64,
    /// 触发后冷却时长 (ms)
    cooldown_ms: u64,
}

impl ShakeDetector {
    /// 使用默认参数创建检测器：
    /// - threshold_squared = 1200²
    /// - required_hits = 2
    /// - window_ms = 300
    /// - cooldown_ms = 700
    pub fn new() -> Self {
        Self {
            last: None,

            hit_count: 0,
            window_start_ms: 0,
            cooldown_until_ms: 0,

            threshold_squared: 1200_i64 * 1200,
            required_hits: 2,
            window_ms: 300,
            cooldown_ms: 700,
        }
    }

    /// 喂入一帧加速度数据，若检测到摇动则返回 `Some(ShakeEvent::Shake)`。
    ///
    /// - `now_ms`: 当前时间戳 (ms)，单调递增
    /// - `x, y, z`: 加速度三轴原始值 (mg)
    pub fn update(&mut self, now_ms: u64, x: i32, y: i32, z: i32) -> Option<ShakeEvent> {
        let current = (x, y, z);

        let Some(previous) = self.last.replace(current) else {
            // 首帧：仅记录，无历史可比较
            return None;
        };

        // 冷却期内静默
        if now_ms < self.cooldown_until_ms {
            return None;
        }

        let motion = motion_squared(current, previous);

        // 当前检测窗口过期 → 重置
        if self.hit_count > 0 && now_ms.saturating_sub(self.window_start_ms) > self.window_ms {
            self.hit_count = 0;
        }

        // 运动幅度不足 → 忽略
        if motion < self.threshold_squared {
            return None;
        }

        // 第一次剧烈运动，开启窗口
        if self.hit_count == 0 {
            self.window_start_ms = now_ms;
        }

        self.hit_count += 1;

        // 命中次数达标 → 触发事件并进入冷却
        if self.hit_count >= self.required_hits {
            self.hit_count = 0;
            self.cooldown_until_ms = now_ms + self.cooldown_ms;
            return Some(ShakeEvent::Shake);
        }

        None
    }
}

impl Default for ShakeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算两帧加速度之间的变化量平方和。
fn motion_squared(a: (i32, i32, i32), b: (i32, i32, i32)) -> i64 {
    let dx = (a.0 - b.0) as i64;
    let dy = (a.1 - b.1) as i64;
    let dz = (a.2 - b.2) as i64;
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 首帧无历史，应返回 None
    #[test]
    fn first_update_returns_none() {
        let mut det = ShakeDetector::new();
        let result = det.update(0, 0, 0, 0);
        assert_eq!(result, None);
    }

    /// 运动幅度低于阈值不触发
    #[test]
    fn motion_below_threshold_returns_none() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧
        let result = det.update(10, 100, 0, 0); // 变化量 = 100² = 10_000 < 1200²
        assert_eq!(result, None);
    }

    /// 窗口内两次剧烈运动触发摇动事件
    #[test]
    fn two_shakes_in_window_triggers_event() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧
        det.update(100, 2000, 0, 0); // hit 1, 变化量 = 2000² > 1200²
        let result = det.update(200, -2000, 0, 0); // hit 2, 变化量 = 4000² > 1200²
        assert_eq!(result, Some(ShakeEvent::Shake));
    }

    /// 触发后进入冷却期，期间不产生事件
    #[test]
    fn cooldown_suppresses_events() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧

        // 触发一次摇动
        det.update(100, 2000, 0, 0);
        let result = det.update(200, -2000, 0, 0);
        assert_eq!(result, Some(ShakeEvent::Shake));

        // 冷却期内 (700ms)，即使剧烈运动也不触发
        let result = det.update(300, 2000, 0, 0);
        assert_eq!(result, None);
        let result = det.update(400, -2000, 0, 0);
        assert_eq!(result, None);
    }

    /// 冷却期结束后可以再次触发
    #[test]
    fn trigger_after_cooldown() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧

        // 第一次触发 @ t=100..200
        det.update(100, 2000, 0, 0);
        let r = det.update(200, -2000, 0, 0);
        assert_eq!(r, Some(ShakeEvent::Shake));

        // 冷却期内静默
        det.update(300, 2000, 0, 0);
        let r = det.update(400, -2000, 0, 0);
        assert_eq!(r, None);

        // 冷却结束 (200 + 700 = 900)，再次触发
        det.update(950, 2000, 0, 0);
        let r = det.update(1050, -2000, 0, 0);
        assert_eq!(r, Some(ShakeEvent::Shake));
    }

    /// 窗口超时未达到命中次数则重置
    #[test]
    fn window_timeout_resets_count() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧

        // hit 1 @ t=100
        det.update(100, 2000, 0, 0);

        // 窗口过期 (window_ms = 300) → 计数重置
        // t=100 + 300 = 400，t=500 时已过期
        det.update(500, -2000, 0, 0); // 开启新窗口，hit 1

        // 需要再一击
        let r = det.update(600, 2000, 0, 0); // hit 2 → 触发
        assert_eq!(r, Some(ShakeEvent::Shake));
    }

    /// 窗口内低幅度运动不影响计数
    #[test]
    fn small_motions_dont_affect_count() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧

        det.update(100, 2000, 0, 0); // hit 1
        det.update(150, 2050, 0, 0); // 变化量 = 50² = 2_500 < 1200²，不影响
        let r = det.update(200, -2000, 0, 0); // hit 2 → 触发
        assert_eq!(r, Some(ShakeEvent::Shake));
    }

    /// 同时到达窗口边界不算过期（now_ms - window_start_ms == window_ms 仍在窗口内）
    #[test]
    fn boundary_exactly_window_ms_is_not_expired() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧

        det.update(100, 2000, 0, 0); // hit 1, window_start = 100
                                     // 恰好 300ms 后 = 400ms，saturating_sub(400, 100) = 300，不大于 300
                                     // 所以窗口未过期
        let r = det.update(400, -2000, 0, 0); // hit 2
        assert_eq!(r, Some(ShakeEvent::Shake));
    }

    /// 单次剧烈运动超窗口后重新开始计数
    #[test]
    fn single_hit_after_window_becomes_new_first_hit() {
        let mut det = ShakeDetector::new();
        det.update(0, 0, 0, 0); // 首帧

        det.update(100, 2000, 0, 0); // hit 1, 开启窗口
                                     // 窗口过期 → hit_count 重置为 0
        det.update(500, -2000, 0, 0); // 新 hit 1

        let r = det.update(600, 2000, 0, 0); // hit 2
        assert_eq!(r, Some(ShakeEvent::Shake));
    }
}
