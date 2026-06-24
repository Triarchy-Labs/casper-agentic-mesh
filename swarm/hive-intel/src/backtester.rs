/// Event-Driven Backtester — ВЕКТОР 5 из OVERKILL Roadmap.
///
/// Прогоняет ИСТОРИЧЕСКИЕ свечи через Paper Engine свеча-за-свечой.
/// Результат: Sharpe, Max Drawdown, Win Rate, PnL curve, Profit Factor.
///
/// Использует:
///   - `paper_engine::PaperEngine` — виртуальные fills с SL/TP
///   - `paper_engine::MarketTick` — формат данных
///   - OHLCV свечи из CSV или прямого ввода
///
/// Простая стратегия для тестирования:
///   - RSI oversold (< 30) → Long
///   - RSI overbought (> 70) → Short
///   - SL = 2% от цены, TP = 4% от цены
use crate::paper_engine::{PaperEngine, MarketTick, Side, PaperStats};

// ════════════════════════════════════════════════════════════════
// OHLCV Candle
// ════════════════════════════════════════════════════════════════

/// Одна OHLCV свеча.
#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp_ms: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// ════════════════════════════════════════════════════════════════
// RSI Calculator (встроенный, zero-alloc для бэктеста)
// ════════════════════════════════════════════════════════════════

/// Инкрементальный RSI (Wilder's smoothing).
pub struct RsiCalculator {
    period: usize,
    avg_gain: f64,
    avg_loss: f64,
    prev_close: Option<f64>,
    count: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
}

impl RsiCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            avg_gain: 0.0,
            avg_loss: 0.0,
            prev_close: None,
            count: 0,
            gains: Vec::with_capacity(period),
            losses: Vec::with_capacity(period),
        }
    }

    /// Обновить RSI с новой ценой. Возвращает RSI (0-100) или None если недостаточно данных.
    pub fn update(&mut self, close: f64) -> Option<f64> {
        if let Some(prev) = self.prev_close {
            let change = close - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { change.abs() } else { 0.0 };

            self.count += 1;

            if self.count <= self.period {
                // Collecting initial period
                self.gains.push(gain);
                self.losses.push(loss);

                if self.count == self.period {
                    self.avg_gain = self.gains.iter().sum::<f64>() / self.period as f64;
                    self.avg_loss = self.losses.iter().sum::<f64>() / self.period as f64;
                }
            } else {
                // Wilder's smoothing
                self.avg_gain = (self.avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
                self.avg_loss = (self.avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;
            }
        }

        self.prev_close = Some(close);

        if self.count >= self.period {
            if self.avg_loss < 1e-10 {
                Some(100.0)
            } else {
                let rs = self.avg_gain / self.avg_loss;
                Some(100.0 - (100.0 / (1.0 + rs)))
            }
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.avg_gain = 0.0;
        self.avg_loss = 0.0;
        self.prev_close = None;
        self.count = 0;
        self.gains.clear();
        self.losses.clear();
    }
}

// ════════════════════════════════════════════════════════════════
// Backtest Config
// ════════════════════════════════════════════════════════════════

/// Конфигурация бэктеста.
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Начальный баланс.
    pub initial_balance: f64,
    /// RSI period.
    pub rsi_period: usize,
    /// RSI порог для Long (oversold).
    pub rsi_oversold: f64,
    /// RSI порог для Short (overbought).
    pub rsi_overbought: f64,
    /// Стоп-лосс в % от цены.
    pub sl_pct: f64,
    /// Тейк-профит в % от цены.
    pub tp_pct: f64,
    /// Размер позиции в % от баланса.
    pub position_size_pct: f64,
    /// Символ.
    pub symbol: String,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_balance: 1000.0,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            sl_pct: 2.0,
            tp_pct: 4.0,
            position_size_pct: 10.0,
            symbol: "BTCUSDT".to_string(),
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Backtest Result
// ════════════════════════════════════════════════════════════════

/// Результат бэктеста.
#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub stats: PaperStats,
    pub equity_curve: Vec<f64>,
    pub candles_processed: usize,
    pub signals_generated: usize,
    pub config: BacktestConfig,
    pub verdict: BacktestVerdict,
}

/// Вердикт бэктеста.
#[derive(Debug, Clone, PartialEq)]
pub enum BacktestVerdict {
    /// Sharpe > 1.5, PF > 1.5, WR > 45% → READY
    Ready,
    /// Sharpe > 0.5, PF > 1.0, WR > 40% → PROMISING
    Promising,
    /// Всё остальное → NOT READY
    NotReady,
}

impl BacktestVerdict {
    pub fn as_str(&self) -> &str {
        match self {
            BacktestVerdict::Ready => "✅ READY — Go live",
            BacktestVerdict::Promising => "🟡 PROMISING — Needs tuning",
            BacktestVerdict::NotReady => "🔴 NOT READY — Do NOT trade",
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Backtester Engine
// ════════════════════════════════════════════════════════════════

/// Event-Driven Backtester.
pub struct Backtester {
    config: BacktestConfig,
    engine: PaperEngine,
    rsi: RsiCalculator,
    equity_curve: Vec<f64>,
    signals: usize,
}

impl Backtester {
    pub fn new(config: BacktestConfig) -> Self {
        let engine = PaperEngine::new(config.initial_balance);
        let rsi = RsiCalculator::new(config.rsi_period);
        Self {
            config,
            engine,
            rsi,
            equity_curve: Vec::new(),
            signals: 0,
        }
    }

    /// Прогнать массив свечей через бэктестер. Свеча за свечой.
    pub fn run(&mut self, candles: &[Candle]) -> BacktestResult {
        for candle in candles {
            self.process_candle(candle);
        }

        // Закрыть все открытые позиции по последней цене
        if let Some(last) = candles.last() {
            let symbols: Vec<String> = self.engine.open_positions.keys().cloned().collect();
            for sym in symbols {
                self.engine.close_position(&sym, last.close, last.timestamp_ms);
            }
        }

        let stats = self.engine.stats();
        let verdict = Self::evaluate_verdict(&stats);

        BacktestResult {
            stats,
            equity_curve: self.equity_curve.clone(),
            candles_processed: candles.len(),
            signals_generated: self.signals,
            config: self.config.clone(),
            verdict,
        }
    }

    /// Обработать одну свечу.
    fn process_candle(&mut self, candle: &Candle) {
        let symbol = &self.config.symbol;

        // 1. Обновить все открытые позиции через on_tick
        let tick = MarketTick {
            symbol: symbol.clone(),
            price: candle.close,
            high: candle.high,
            low: candle.low,
            volume: candle.volume,
            timestamp_ms: candle.timestamp_ms,
        };
        self.engine.on_tick(&tick);

        // 2. Записать equity
        self.equity_curve.push(self.engine.equity);

        // 3. Рассчитать RSI
        let rsi_val = match self.rsi.update(candle.close) {
            Some(v) => v,
            None => return, // Недостаточно данных
        };

        // 4. Генерировать сигнал (только если нет открытой позиции)
        if self.engine.open_positions.contains_key(symbol) || self.engine.is_circuit_broken {
            return;
        }

        // Position sizing
        let qty = (self.engine.balance * self.config.position_size_pct / 100.0) / candle.close;
        if qty < 1e-8 { return; }

        // RSI oversold → Long
        if rsi_val < self.config.rsi_oversold {
            let sl = candle.close * (1.0 - self.config.sl_pct / 100.0);
            let tp = candle.close * (1.0 + self.config.tp_pct / 100.0);
            let _ = self.engine.open_position(
                symbol, Side::Long, candle.close, qty,
                Some(sl), Some(tp), candle.timestamp_ms,
            );
            self.signals += 1;
        }
        // RSI overbought → Short
        else if rsi_val > self.config.rsi_overbought {
            let sl = candle.close * (1.0 + self.config.sl_pct / 100.0);
            let tp = candle.close * (1.0 - self.config.tp_pct / 100.0);
            let _ = self.engine.open_position(
                symbol, Side::Short, candle.close, qty,
                Some(sl), Some(tp), candle.timestamp_ms,
            );
            self.signals += 1;
        }
    }

    /// Оценить вердикт по статистике.
    fn evaluate_verdict(stats: &PaperStats) -> BacktestVerdict {
        if stats.total_trades < 10 {
            return BacktestVerdict::NotReady;
        }

        let sharpe_ok = stats.sharpe_ratio > 1.5;
        let pf_ok = stats.profit_factor > 1.5;
        let wr_ok = stats.win_rate > 0.45;

        if sharpe_ok && pf_ok && wr_ok {
            BacktestVerdict::Ready
        } else if stats.sharpe_ratio > 0.5 && stats.profit_factor > 1.0 && stats.win_rate > 0.40 {
            BacktestVerdict::Promising
        } else {
            BacktestVerdict::NotReady
        }
    }

    /// Красивый отчёт.
    pub fn print_report(result: &BacktestResult) {
        println!("\n╔══════════════════════════════════════════════════╗");
        println!("║        📊 BACKTEST REPORT — {}        ", result.config.symbol);
        println!("╠══════════════════════════════════════════════════╣");
        println!("║ Candles:     {:<10}                         ║", result.candles_processed);
        println!("║ Signals:     {:<10}                         ║", result.signals_generated);
        println!("║ Trades:      {} ({} W / {} L)              ",
            result.stats.total_trades, result.stats.wins, result.stats.losses);
        println!("║ Win Rate:    {:.1}%                              ",
            result.stats.win_rate * 100.0);
        println!("║ Total PnL:   ${:.2}                           ",
            result.stats.total_pnl);
        println!("║ Sharpe:      {:.2}                              ",
            result.stats.sharpe_ratio);
        println!("║ Profit F:    {:.2}                              ",
            result.stats.profit_factor);
        println!("║ Max DD:      ${:.2}                           ",
            result.stats.max_drawdown);
        println!("║ Avg Win:     ${:.2}                           ",
            result.stats.avg_win);
        println!("║ Avg Loss:    ${:.2}                           ",
            result.stats.avg_loss);
        println!("╠══════════════════════════════════════════════════╣");
        println!("║ VERDICT:     {}  ", result.verdict.as_str());
        println!("╚══════════════════════════════════════════════════╝\n");
    }
}

/// Загрузить свечи из CSV файла.
/// Формат: timestamp_ms,open,high,low,close,volume
pub fn load_candles_from_csv(path: &str) -> Result<Vec<Candle>, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {path}: {e}"))?;

    let mut candles = Vec::new();

    for (i, line) in data.lines().enumerate() {
        // Skip header
        if i == 0 && line.contains("timestamp") { continue; }
        let line = line.trim();
        if line.is_empty() { continue; }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            continue;
        }

        let candle = Candle {
            timestamp_ms: parts[0].parse().unwrap_or(0),
            open: parts[1].parse().unwrap_or(0.0),
            high: parts[2].parse().unwrap_or(0.0),
            low: parts[3].parse().unwrap_or(0.0),
            close: parts[4].parse().unwrap_or(0.0),
            volume: parts[5].parse().unwrap_or(0.0),
        };

        if candle.close > 0.0 {
            candles.push(candle);
        }
    }

    if candles.is_empty() {
        Err("No valid candles found".to_string())
    } else {
        Ok(candles)
    }
}

/// Сгенерировать синтетические свечи для тестирования.
/// Создаёт волнообразный рынок (синусоида + шум).
pub fn generate_synthetic_candles(
    base_price: f64,
    count: usize,
    interval_ms: i64,
    amplitude_pct: f64,
) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let mut ts = 1_700_000_000_000_i64; // Nov 2023 epoch

    // Простой детерминистический PRNG (чтобы тесты были стабильные)
    let mut seed: u64 = 42;
    let next_rand = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*s >> 33) as f64) / (u32::MAX as f64) - 0.5 // [-0.5, 0.5)
    };

    for i in 0..count {
        let wave = (i as f64 * 0.05).sin() * amplitude_pct / 100.0;
        let noise = next_rand(&mut seed) * 0.005; // ±0.5% noise
        let price = base_price * (1.0 + wave + noise);

        let spread = base_price * 0.002; // 0.2% spread
        let high = price + spread * (0.5 + next_rand(&mut seed).abs());
        let low = price - spread * (0.5 + next_rand(&mut seed).abs());
        let open = price + next_rand(&mut seed) * spread * 0.5;

        candles.push(Candle {
            timestamp_ms: ts,
            open,
            high,
            low,
            close: price,
            volume: 100.0 + next_rand(&mut seed).abs() * 900.0,
        });

        ts += interval_ms;
    }

    candles
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_initial_period() {
        let mut rsi = RsiCalculator::new(14);
        // Need 14 changes (15 prices) to get first RSI
        // Use alternating prices so RSI isn't pinned at extremes
        let prices = [100.0, 101.0, 99.5, 102.0, 98.0, 103.0, 97.0, 101.5, 99.0, 102.5, 98.5, 100.5, 99.0, 101.0];
        for &p in &prices {
            assert!(rsi.update(p).is_none(), "Should not have RSI yet");
        }
        let val = rsi.update(100.0);
        assert!(val.is_some(), "Should have RSI after 14+1 prices");
        let v = val.unwrap();
        assert!(v > 0.0 && v < 100.0, "RSI should be between 0 and 100 exclusive, got {}", v);
    }

    #[test]
    fn test_rsi_all_up() {
        let mut rsi = RsiCalculator::new(5);
        for i in 0..20 {
            rsi.update(100.0 + i as f64);
        }
        let val = rsi.update(120.0).unwrap();
        assert!(val > 90.0, "All-up market should have RSI > 90, got {:.1}", val);
    }

    #[test]
    fn test_rsi_all_down() {
        let mut rsi = RsiCalculator::new(5);
        for i in 0..20 {
            rsi.update(200.0 - i as f64);
        }
        let val = rsi.update(179.0).unwrap();
        assert!(val < 10.0, "All-down market should have RSI < 10, got {:.1}", val);
    }

    #[test]
    fn test_rsi_reset() {
        let mut rsi = RsiCalculator::new(5);
        for i in 0..10 { rsi.update(100.0 + i as f64); }
        rsi.reset();
        assert!(rsi.update(100.0).is_none(), "After reset, should need new data");
    }

    #[test]
    fn test_synthetic_candles() {
        let candles = generate_synthetic_candles(100.0, 500, 60_000, 5.0);
        assert_eq!(candles.len(), 500);
        for c in &candles {
            assert!(c.high >= c.low);
            assert!(c.close > 0.0);
            assert!(c.volume > 0.0);
        }
    }

    #[test]
    fn test_synthetic_candles_deterministic() {
        let c1 = generate_synthetic_candles(100.0, 100, 60_000, 5.0);
        let c2 = generate_synthetic_candles(100.0, 100, 60_000, 5.0);
        for (a, b) in c1.iter().zip(c2.iter()) {
            assert!((a.close - b.close).abs() < 1e-10, "Should be deterministic");
        }
    }

    #[test]
    fn test_backtest_runs_to_completion() {
        let candles = generate_synthetic_candles(100.0, 500, 60_000, 10.0);
        let config = BacktestConfig {
            initial_balance: 1000.0,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            sl_pct: 2.0,
            tp_pct: 4.0,
            position_size_pct: 10.0,
            symbol: "TESTUSDT".to_string(),
        };

        let mut bt = Backtester::new(config);
        let result = bt.run(&candles);

        assert_eq!(result.candles_processed, 500);
        assert_eq!(result.equity_curve.len(), 500);
        assert!(result.stats.current_equity > 0.0, "Should not go bankrupt on synthetic data");
    }

    #[test]
    fn test_backtest_generates_signals() {
        // Wide oscillation → should trigger RSI signals
        let candles = generate_synthetic_candles(100.0, 1000, 60_000, 15.0);
        let config = BacktestConfig {
            initial_balance: 1000.0,
            rsi_period: 10,
            rsi_oversold: 35.0,
            rsi_overbought: 65.0,
            sl_pct: 3.0,
            tp_pct: 6.0,
            position_size_pct: 10.0,
            symbol: "WAVE".to_string(),
        };

        let mut bt = Backtester::new(config);
        let result = bt.run(&candles);

        assert!(result.signals_generated > 0, "Should generate signals on volatile data");
        assert!(result.stats.total_trades > 0, "Should have completed trades");
    }

    #[test]
    fn test_backtest_verdict_not_ready() {
        let stats = PaperStats {
            total_trades: 5, wins: 2, losses: 3,
            win_rate: 0.4, total_pnl: -10.0,
            max_drawdown: 50.0, peak_equity: 1000.0,
            current_equity: 950.0, sharpe_ratio: -0.5,
            profit_factor: 0.8, avg_win: 5.0, avg_loss: -6.67,
            best_trade: 8.0, worst_trade: -10.0,
            session_start_ms: 0, session_duration_ms: 1000,
        };
        assert_eq!(Backtester::evaluate_verdict(&stats), BacktestVerdict::NotReady);
    }

    #[test]
    fn test_backtest_verdict_ready() {
        let stats = PaperStats {
            total_trades: 50, wins: 30, losses: 20,
            win_rate: 0.6, total_pnl: 200.0,
            max_drawdown: 30.0, peak_equity: 1200.0,
            current_equity: 1200.0, sharpe_ratio: 2.0,
            profit_factor: 2.5, avg_win: 10.0, avg_loss: -5.0,
            best_trade: 30.0, worst_trade: -15.0,
            session_start_ms: 0, session_duration_ms: 1000,
        };
        assert_eq!(Backtester::evaluate_verdict(&stats), BacktestVerdict::Ready);
    }

    #[test]
    fn test_backtest_verdict_promising() {
        let stats = PaperStats {
            total_trades: 30, wins: 15, losses: 15,
            win_rate: 0.5, total_pnl: 50.0,
            max_drawdown: 40.0, peak_equity: 1050.0,
            current_equity: 1050.0, sharpe_ratio: 0.8,
            profit_factor: 1.2, avg_win: 8.0, avg_loss: -5.0,
            best_trade: 20.0, worst_trade: -12.0,
            session_start_ms: 0, session_duration_ms: 1000,
        };
        assert_eq!(Backtester::evaluate_verdict(&stats), BacktestVerdict::Promising);
    }

    #[test]
    fn test_backtest_equity_curve_monotonic_start() {
        let candles = generate_synthetic_candles(100.0, 100, 60_000, 3.0);
        let config = BacktestConfig::default();
        let mut bt = Backtester::new(config);
        let result = bt.run(&candles);

        // First equity point should be initial balance (no trades yet)
        assert!(
            (result.equity_curve[0] - 1000.0).abs() < 1e-10,
            "First equity point should be initial balance"
        );
    }

    #[test]
    fn test_csv_loading_invalid_path() {
        let result = load_candles_from_csv("/nonexistent/path.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_default() {
        let cfg = BacktestConfig::default();
        assert_eq!(cfg.initial_balance, 1000.0);
        assert_eq!(cfg.rsi_period, 14);
        assert_eq!(cfg.symbol, "BTCUSDT");
    }

    #[test]
    fn test_backtest_empty_candles() {
        let config = BacktestConfig::default();
        let mut bt = Backtester::new(config);
        let result = bt.run(&[]);

        assert_eq!(result.candles_processed, 0);
        assert_eq!(result.signals_generated, 0);
        assert_eq!(result.stats.total_trades, 0);
    }
}
