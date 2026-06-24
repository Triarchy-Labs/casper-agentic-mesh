use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ── ATR-Based Stop Calculator ──────────────────────────────────────────
/// Average True Range stop/target calculator.
/// Absorbed from Swarmbots RISK-MODEL: Gate 1.
#[derive(Debug, Clone)]
pub struct AtrStops {
    pub stop_distance: f64,
    pub target_distance: f64,
    pub entry_price: f64,
}

impl AtrStops {
    /// Calculate stop and target distances from ATR(14, 1h).
    /// - Stop  = 1.5 × ATR
    /// - Target = 2 × stop (reward/risk = 2:1)
    /// - Floor: stop >= 0.4% of entry
    /// - Ceiling: stop <= 3% of entry
    pub fn calculate(atr_14: f64, entry_price: f64) -> Self {
        let raw_stop = 1.5 * atr_14;
        let floor = entry_price * 0.004;
        let ceiling = entry_price * 0.03;
        let stop_distance = raw_stop.max(floor).min(ceiling);
        let target_distance = stop_distance * 2.0;
        Self { stop_distance, target_distance, entry_price }
    }

    pub fn stop_price_long(&self) -> f64 { self.entry_price - self.stop_distance }
    pub fn target_price_long(&self) -> f64 { self.entry_price + self.target_distance }
    pub fn stop_price_short(&self) -> f64 { self.entry_price + self.stop_distance }
    pub fn target_price_short(&self) -> f64 { self.entry_price - self.target_distance }
}

// ── Kelly Sizing ───────────────────────────────────────────────────────
/// Kelly Criterion position sizer with regime modulation.
/// Absorbed from Swarmbots RISK-MODEL: Gate 2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarketRegime {
    Calm,
    Trending,
    Choppy,
}

impl MarketRegime {
    pub fn modulator(&self) -> f64 {
        match self {
            MarketRegime::Calm => 1.0,
            MarketRegime::Trending => 0.85,
            MarketRegime::Choppy => 0.4,
        }
    }
}

/// Calculate position size using half-Kelly with regime modulation.
/// `win_rate`: 7-day rolling win rate (0.0–1.0)
/// `avg_loss_r`: average loss in R-multiples (positive number)
/// `bankroll`: current available capital
/// `regime`: current market regime
pub fn kelly_size(win_rate: f64, avg_loss_r: f64, bankroll: f64, regime: MarketRegime) -> f64 {
    if avg_loss_r <= 0.0 || win_rate <= 0.0 {
        return 0.0;
    }
    let raw_kelly = (2.0 * win_rate - 1.0).max(0.0) / avg_loss_r;
    let clamped = raw_kelly.clamp(0.1, 0.6);
    bankroll * clamped * regime.modulator()
}

// ── Bucket-Cap Correlation Guard ───────────────────────────────────────
/// Prevents simultaneous exposure to correlated assets.
/// Absorbed from Swarmbots RISK-MODEL: Gate 3.
#[derive(Debug, Clone)]
pub struct BucketCapGuard {
    /// Maps bucket name -> (max simultaneous positions, current open symbols)
    buckets: HashMap<String, (usize, Vec<String>)>,
    /// Maps symbol -> bucket name
    symbol_map: HashMap<String, String>,
}

impl Default for BucketCapGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl BucketCapGuard {
    pub fn new() -> Self {
        let mut buckets = HashMap::new();
        let mut symbol_map = HashMap::new();

        // Default Mantle DeFi asset correlation buckets
        // Bucket A: Major L1/L2 (BTC-correlated)
        buckets.insert("major".to_string(), (1, Vec::new()));
        for sym in &["WBTC", "WETH", "MNT"] {
            symbol_map.insert(sym.to_string(), "major".to_string());
        }

        // Bucket B: DeFi bluechips
        buckets.insert("defi".to_string(), (1, Vec::new()));
        for sym in &["UNI", "AAVE", "LINK"] {
            symbol_map.insert(sym.to_string(), "defi".to_string());
        }

        // Bucket C: Stablecoins (independent — high cap since they're hedges)
        buckets.insert("stable".to_string(), (3, Vec::new()));
        for sym in &["USDT", "USDC", "DAI"] {
            symbol_map.insert(sym.to_string(), "stable".to_string());
        }

        Self { buckets, symbol_map }
    }

    /// Check if opening a position on `symbol` would breach bucket cap.
    pub fn can_open(&self, symbol: &str) -> bool {
        if let Some(bucket_name) = self.symbol_map.get(symbol)
            && let Some((max, open)) = self.buckets.get(bucket_name)
        {
            return open.len() < *max;
        }
        // Unknown symbols are unconstrained
        true
    }

    /// Register an opened position.
    pub fn register_open(&mut self, symbol: &str) {
        if let Some(bucket_name) = self.symbol_map.get(symbol).cloned()
            && let Some((_max, open)) = self.buckets.get_mut(&bucket_name)
            && !open.contains(&symbol.to_string())
        {
            open.push(symbol.to_string());
        }
    }

    /// Unregister a closed position.
    pub fn register_close(&mut self, symbol: &str) {
        if let Some(bucket_name) = self.symbol_map.get(symbol).cloned()
            && let Some((_max, open)) = self.buckets.get_mut(&bucket_name)
        {
            open.retain(|s| s != symbol);
        }
    }
}

// ── Kill Switch ────────────────────────────────────────────────────────
/// Dual-trigger circuit breaker: consecutive losses OR daily PnL breach.
/// Absorbed from Swarmbots RISK-MODEL: Kill-switch.
#[derive(Debug, Clone)]
pub struct KillSwitch {
    pub consecutive_losses: u32,
    pub max_consecutive_losses: u32,
    pub daily_pnl: f64,
    pub daily_pnl_floor_pct: f64, // e.g., -0.02 for -2%
    pub bankroll: f64,
    pub is_triggered: bool,
    pub triggered_at: Option<u64>,
    pub cooldown_seconds: u64,
}

impl KillSwitch {
    pub fn new(bankroll: f64) -> Self {
        Self {
            consecutive_losses: 0,
            max_consecutive_losses: 3,
            daily_pnl: 0.0,
            daily_pnl_floor_pct: -0.02,
            bankroll,
            is_triggered: false,
            triggered_at: None,
            cooldown_seconds: 6 * 3600, // 6 hours default
        }
    }

    /// Record a trade result. Returns true if kill-switch was triggered.
    pub fn record_trade(&mut self, pnl: f64) -> bool {
        self.daily_pnl += pnl;

        if pnl < 0.0 {
            self.consecutive_losses += 1;
        } else {
            self.consecutive_losses = 0;
        }

        // Check triggers
        let consecutive_breach = self.consecutive_losses >= self.max_consecutive_losses;
        let pnl_breach = self.daily_pnl <= self.bankroll * self.daily_pnl_floor_pct;

        if consecutive_breach || pnl_breach {
            self.is_triggered = true;
            self.triggered_at = Some(now_epoch());
            println!("[KillSwitch] TRIGGERED! Consecutive losses: {}, Daily PnL: {:.4}",
                     self.consecutive_losses, self.daily_pnl);
        }

        self.is_triggered
    }

    /// Check if the system is currently paused by kill-switch.
    pub fn is_paused(&self) -> bool {
        if !self.is_triggered {
            return false;
        }
        if let Some(triggered_at) = self.triggered_at {
            let elapsed = now_epoch() - triggered_at;
            if elapsed >= self.cooldown_seconds {
                // Auto re-arm after cooldown
                return false;
            }
        }
        true
    }

    /// Reset daily PnL counter (call at start of each trading day).
    pub fn reset_daily(&mut self) {
        self.daily_pnl = 0.0;
        if !self.is_paused() {
            self.is_triggered = false;
            self.triggered_at = None;
            self.consecutive_losses = 0;
        }
    }
}

// ── Cooldown Gate ──────────────────────────────────────────────────────
/// Per-symbol cooldown tracker to prevent overtrading.
/// Absorbed from Swarmbots RISK-MODEL: Gate 4.
#[derive(Debug, Clone, Default)]
pub struct CooldownGate {
    /// symbol -> epoch timestamp when cooldown expires
    cooldowns: HashMap<String, u64>,
    pub default_cooldown_seconds: u64,
}

impl CooldownGate {
    pub fn new(cooldown_seconds: u64) -> Self {
        Self {
            cooldowns: HashMap::new(),
            default_cooldown_seconds: cooldown_seconds,
        }
    }

    /// Check if a symbol is clear to trade.
    pub fn is_clear(&self, symbol: &str) -> bool {
        if let Some(expires_at) = self.cooldowns.get(symbol) {
            return now_epoch() >= *expires_at;
        }
        true
    }

    /// Start cooldown for a symbol after a trade closes.
    pub fn start_cooldown(&mut self, symbol: &str) {
        let expires = now_epoch() + self.default_cooldown_seconds;
        self.cooldowns.insert(symbol.to_string(), expires);
    }
}

// ── Auto-Ramp (5-Gate Capital Scaling) ─────────────────────────────────
/// Deterministic state machine that scales capital through stages.
/// Absorbed from Swarmbots AUTO-RAMP.md.
#[derive(Debug, Clone)]
pub struct AutoRampStage {
    pub phase: u8,
    pub bankroll_cap: f64,
    pub per_trade_size: f64,
    pub daily_kill_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct AutoRamp {
    pub current_stage: u8,
    pub stages: Vec<AutoRampStage>,
    pub last_promotion_epoch: u64,
    pub promotion_cooldown: u64, // 168 hours = 7 days
}

impl Default for AutoRamp {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoRamp {
    pub fn new() -> Self {
        let stages = vec![
            AutoRampStage { phase: 0, bankroll_cap: 30.0,   per_trade_size: 10.0,  daily_kill_threshold: -1.5 },
            AutoRampStage { phase: 1, bankroll_cap: 300.0,  per_trade_size: 25.0,  daily_kill_threshold: -15.0 },
            AutoRampStage { phase: 2, bankroll_cap: 750.0,  per_trade_size: 50.0,  daily_kill_threshold: -30.0 },
            AutoRampStage { phase: 3, bankroll_cap: 2000.0, per_trade_size: 100.0, daily_kill_threshold: -80.0 },
            AutoRampStage { phase: 4, bankroll_cap: 5000.0, per_trade_size: 250.0, daily_kill_threshold: -200.0 },
        ];
        Self {
            current_stage: 0,
            stages,
            last_promotion_epoch: 0,
            promotion_cooldown: 168 * 3600, // 7 days
        }
    }

    pub fn current(&self) -> &AutoRampStage {
        &self.stages[self.current_stage as usize]
    }

    /// Evaluate all 5 gates for promotion.
    pub fn evaluate_promotion(
        &mut self,
        closed_trades_96h: usize,
        pnl_7d: f64,
        kill_switch_incidents_96h: usize,
        bucket_breaches_96h: usize,
    ) -> bool {
        let next = self.current_stage as usize + 1;
        if next >= self.stages.len() {
            return false; // Max stage reached
        }

        // Gate 1: Trade volume
        if closed_trades_96h < 10 { return false; }
        // Gate 2: 7-day PnL positive
        if pnl_7d <= 0.0 { return false; }
        // Gate 3: No kill-switch incidents
        if kill_switch_incidents_96h > 0 { return false; }
        // Gate 4: Bucket-cap discipline
        if bucket_breaches_96h > 1 { return false; }
        // Gate 5: Cooldown since last promotion
        let elapsed = now_epoch() - self.last_promotion_epoch;
        if elapsed < self.promotion_cooldown { return false; }

        // All gates passed — promote
        self.current_stage += 1;
        self.last_promotion_epoch = now_epoch();
        println!("[AutoRamp] PROMOTED to Phase {}! New bankroll cap: {}",
                 self.current_stage, self.current().bankroll_cap);
        true
    }

    /// Hard demote: kill-switch triggered → drop one stage.
    pub fn hard_demote(&mut self) {
        if self.current_stage > 0 {
            self.current_stage -= 1;
            println!("[AutoRamp] HARD DEMOTE to Phase {}!", self.current_stage);
        }
    }
}

// ── Unified Risk Gate ──────────────────────────────────────────────────
/// Single entry point that runs all pre-order guardrails.
/// Returns `Ok(position_size)` if trade is allowed, `Err(reason)` if blocked.
pub struct RiskGate {
    pub kill_switch: KillSwitch,
    pub bucket_guard: BucketCapGuard,
    pub cooldown: CooldownGate,
    pub auto_ramp: AutoRamp,
}

impl RiskGate {
    pub fn new(bankroll: f64) -> Self {
        Self {
            kill_switch: KillSwitch::new(bankroll),
            bucket_guard: BucketCapGuard::new(),
            cooldown: CooldownGate::new(30 * 60), // 30 min default
            auto_ramp: AutoRamp::new(),
        }
    }

    /// Run all pre-trade risk checks. Returns position size if approved.
    pub fn evaluate(
        &self,
        symbol: &str,
        win_rate: f64,
        avg_loss_r: f64,
        regime: MarketRegime,
    ) -> Result<f64, String> {
        // Gate 1: Kill-switch
        if self.kill_switch.is_paused() {
            return Err("BLOCKED: Kill-switch active — system paused".to_string());
        }

        // Gate 2: Cooldown
        if !self.cooldown.is_clear(symbol) {
            return Err(format!("BLOCKED: {} in cooldown period", symbol));
        }

        // Gate 3: Bucket-cap correlation
        if !self.bucket_guard.can_open(symbol) {
            return Err(format!("BLOCKED: Bucket cap reached for {}", symbol));
        }

        // Gate 4: Kelly sizing (capped by auto-ramp per-trade limit)
        let kelly = kelly_size(win_rate, avg_loss_r, self.auto_ramp.current().bankroll_cap, regime);
        let capped = kelly.min(self.auto_ramp.current().per_trade_size);

        if capped < 1.0 {
            return Err("BLOCKED: Kelly sizing too small — edge insufficient".to_string());
        }

        Ok(capped)
    }
}

// ── Legacy API (backward compat) ───────────────────────────────────────
pub fn is_exposure_safe(current_exposure: f64, max_exposure: f64) -> bool {
    current_exposure < max_exposure
}

// ── Helpers ────────────────────────────────────────────────────────────
fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atr_stops() {
        let stops = AtrStops::calculate(50.0, 3000.0);
        assert!((stops.stop_distance - 75.0).abs() < 0.01);
        assert!((stops.target_distance - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_atr_floor() {
        // ATR so small it hits the floor
        let stops = AtrStops::calculate(0.1, 3000.0);
        assert!((stops.stop_distance - 12.0).abs() < 0.01); // 0.4% of 3000
    }

    #[test]
    fn test_kelly_sizing() {
        let size = kelly_size(0.65, 1.0, 1000.0, MarketRegime::Calm);
        // kelly = (2*0.65 - 1) / 1.0 = 0.3, clamped to [0.1, 0.6] = 0.3
        // size = 1000 * 0.3 * 1.0 = 300
        assert!((size - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_kelly_choppy() {
        let size = kelly_size(0.65, 1.0, 1000.0, MarketRegime::Choppy);
        // 300 * 0.4 = 120
        assert!((size - 120.0).abs() < 0.01);
    }

    #[test]
    fn test_bucket_guard() {
        let mut guard = BucketCapGuard::new();
        assert!(guard.can_open("WBTC"));
        guard.register_open("WBTC");
        // Same bucket — should block
        assert!(!guard.can_open("WETH"));
        // Close WBTC — WETH should be available again
        guard.register_close("WBTC");
        assert!(guard.can_open("WETH"));
    }

    #[test]
    fn test_kill_switch_consecutive() {
        let mut ks = KillSwitch::new(1000.0);
        ks.record_trade(-5.0);
        ks.record_trade(-5.0);
        assert!(!ks.is_triggered);
        ks.record_trade(-5.0); // 3rd consecutive loss
        assert!(ks.is_triggered);
    }

    #[test]
    fn test_kill_switch_pnl() {
        let mut ks = KillSwitch::new(1000.0);
        // -2% of 1000 = -20
        ks.record_trade(-25.0); // Single big loss
        assert!(ks.is_triggered);
    }

    #[test]
    fn test_risk_gate_full() {
        let gate = RiskGate::new(300.0);
        let result = gate.evaluate("WBTC", 0.6, 1.0, MarketRegime::Calm);
        assert!(result.is_ok());
    }
}
