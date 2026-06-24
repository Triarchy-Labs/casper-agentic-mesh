/// Paper Trading Engine — Симуляция торговли на РЕАЛЬНЫХ данных с ВИРТУАЛЬНЫМ балансом.
///
/// DONOR: Концепция из nautilus_trader (SimulatedExchange).
/// Адаптировано для Hive Mind V10 architecture.
///
/// Правило: НА БИРЖУ С РЕАЛЬНЫМИ ДЕНЬГАМИ — ТОЛЬКО ПОСЛЕ 2 НЕДЕЛЬ paper trading В ПЛЮСЕ.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════
// Core Types
// ════════════════════════════════════════════════════════════════

/// Направление сделки.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Long,
    Short,
}

/// Статус ордера.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    StopLoss,
    TakeProfit,
}

/// Один виртуальный ордер.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperOrder {
    pub id: u64,
    pub symbol: String,
    pub side: Side,
    pub entry_price: f64,
    pub quantity: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub status: OrderStatus,
    pub pnl: f64,
    pub opened_at_ms: i64,
    pub closed_at_ms: Option<i64>,
}

/// Снимок рыночных данных для симуляции.
#[derive(Debug, Clone)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub timestamp_ms: i64,
}

/// Статистика paper trading сессии.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperStats {
    pub total_trades: u64,
    pub wins: u64,
    pub losses: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub max_drawdown: f64,
    pub peak_equity: f64,
    pub current_equity: f64,
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
    pub session_start_ms: i64,
    pub session_duration_ms: i64,
}

// ════════════════════════════════════════════════════════════════
// Paper Trading Engine
// ════════════════════════════════════════════════════════════════

/// Движок виртуальной торговли.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperEngine {
    pub initial_balance: f64,
    pub balance: f64,
    pub equity: f64,
    pub peak_equity: f64,
    pub max_drawdown: f64,
    pub open_positions: HashMap<String, PaperOrder>,
    pub closed_trades: Vec<PaperOrder>,
    pub pnl_history: Vec<f64>,
    next_order_id: u64,
    session_start_ms: i64,
    /// Максимальная просадка в % при которой торговля БЛОКИРУЕТСЯ.
    pub circuit_breaker_pct: f64,
    /// Максимальный дневной убыток после которого торговля БЛОКИРУЕТСЯ.
    pub daily_loss_limit: f64,
    pub daily_loss: f64,
    pub is_circuit_broken: bool,
}

impl PaperEngine {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            balance: initial_balance,
            equity: initial_balance,
            peak_equity: initial_balance,
            max_drawdown: 0.0,
            open_positions: HashMap::new(),
            closed_trades: Vec::new(),
            pnl_history: Vec::new(),
            next_order_id: 1,
            session_start_ms: 0,
            circuit_breaker_pct: 10.0,  // Стоп при -10% от пика
            daily_loss_limit: 50.0,     // Стоп при -$50 за день
            daily_loss: 0.0,
            is_circuit_broken: false,
        }
    }

    /// Открыть виртуальную позицию.
    pub fn open_position(
        &mut self,
        symbol: &str,
        side: Side,
        price: f64,
        quantity: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
        timestamp_ms: i64,
    ) -> Result<u64, String> {
        // Circuit breaker check
        if self.is_circuit_broken {
            return Err("🚫 CIRCUIT BREAKER: торговля заблокирована".to_string());
        }

        // Проверка — уже есть открытая позиция по этому символу?
        if self.open_positions.contains_key(symbol) {
            return Err(format!("⚠️ Уже есть открытая позиция по {symbol}"));
        }

        // Проверка маржи
        let notional = price * quantity;
        if notional > self.balance * 0.5 {
            return Err(format!("⚠️ Размер позиции ${:.2} > 50% баланса ${:.2}", 
                notional, self.balance));
        }

        if self.session_start_ms == 0 {
            self.session_start_ms = timestamp_ms;
        }

        let order = PaperOrder {
            id: self.next_order_id,
            symbol: symbol.to_string(),
            side,
            entry_price: price,
            quantity,
            stop_loss,
            take_profit,
            status: OrderStatus::Filled,
            pnl: 0.0,
            opened_at_ms: timestamp_ms,
            closed_at_ms: None,
        };

        self.next_order_id += 1;
        let id = order.id;
        self.open_positions.insert(symbol.to_string(), order);

        println!("  📝 [PAPER] OPEN {side:?} {symbol} @ ${price:.2} qty={quantity:.4} SL={stop_loss:?} TP={take_profit:?}");

        Ok(id)
    }

    /// Обновить все открытые позиции по текущей рыночной цене.
    /// Проверяет SL/TP и закрывает если сработали.
    pub fn on_tick(&mut self, tick: &MarketTick) {
        let symbol = &tick.symbol;

        if let Some(mut order) = self.open_positions.remove(symbol) {
            // Считаем unrealized PnL
            let unrealized = match order.side {
                Side::Long => (tick.price - order.entry_price) * order.quantity,
                Side::Short => (order.entry_price - tick.price) * order.quantity,
            };

            // Проверка Stop Loss (используем high/low для реалистичности)
            if let Some(sl) = order.stop_loss {
                let hit = match order.side {
                    Side::Long => tick.low <= sl,
                    Side::Short => tick.high >= sl,
                };
                if hit {
                    let sl_pnl = match order.side {
                        Side::Long => (sl - order.entry_price) * order.quantity,
                        Side::Short => (order.entry_price - sl) * order.quantity,
                    };
                    self.close_order(&mut order, sl_pnl, OrderStatus::StopLoss, tick.timestamp_ms);
                    return;
                }
            }

            // Проверка Take Profit
            if let Some(tp) = order.take_profit {
                let hit = match order.side {
                    Side::Long => tick.high >= tp,
                    Side::Short => tick.low <= tp,
                };
                if hit {
                    let tp_pnl = match order.side {
                        Side::Long => (tp - order.entry_price) * order.quantity,
                        Side::Short => (order.entry_price - tp) * order.quantity,
                    };
                    self.close_order(&mut order, tp_pnl, OrderStatus::TakeProfit, tick.timestamp_ms);
                    return;
                }
            }

            // Позиция всё ещё открыта — обновляем equity
            order.pnl = unrealized;
            self.open_positions.insert(symbol.clone(), order);
        }

        self.update_equity();
    }

    /// Закрыть позицию вручную по рыночной цене.
    pub fn close_position(&mut self, symbol: &str, price: f64, timestamp_ms: i64) -> Option<f64> {
        if let Some(mut order) = self.open_positions.remove(symbol) {
            let pnl = match order.side {
                Side::Long => (price - order.entry_price) * order.quantity,
                Side::Short => (order.entry_price - price) * order.quantity,
            };
            self.close_order(&mut order, pnl, OrderStatus::Filled, timestamp_ms);
            Some(pnl)
        } else {
            None
        }
    }

    /// Внутренний метод закрытия ордера.
    fn close_order(&mut self, order: &mut PaperOrder, pnl: f64, status: OrderStatus, timestamp_ms: i64) {
        order.pnl = pnl;
        order.status = status.clone();
        order.closed_at_ms = Some(timestamp_ms);

        self.balance += pnl;
        self.pnl_history.push(pnl);

        // Трекинг дневного убытка
        if pnl < 0.0 {
            self.daily_loss += pnl.abs();
        }

        let emoji = if pnl >= 0.0 { "✅" } else { "❌" };
        println!("  {} [PAPER] CLOSE {} {:?} PnL=${:.2} | Balance=${:.2}",
            emoji, order.symbol, status, pnl, self.balance);

        self.closed_trades.push(order.clone());
        self.update_equity();

        // Circuit breaker checks
        let dd_pct = if self.peak_equity > 0.0 {
            ((self.peak_equity - self.equity) / self.peak_equity) * 100.0
        } else { 0.0 };

        if dd_pct >= self.circuit_breaker_pct {
            self.is_circuit_broken = true;
            println!("  🚨 [PAPER] CIRCUIT BREAKER! Drawdown {:.1}% >= {:.1}% limit",
                dd_pct, self.circuit_breaker_pct);
        }

        if self.daily_loss >= self.daily_loss_limit {
            self.is_circuit_broken = true;
            println!("  🚨 [PAPER] DAILY LOSS LIMIT! ${:.2} >= ${:.2} limit",
                self.daily_loss, self.daily_loss_limit);
        }
    }

    /// Обновить equity (баланс + unrealized PnL).
    fn update_equity(&mut self) {
        let unrealized: f64 = self.open_positions.values().map(|o| o.pnl).sum();
        self.equity = self.balance + unrealized;

        if self.equity > self.peak_equity {
            self.peak_equity = self.equity;
        }

        let dd = self.peak_equity - self.equity;
        if dd > self.max_drawdown {
            self.max_drawdown = dd;
        }
    }

    /// Сбросить дневные лимиты (вызывать в начале нового дня).
    pub fn reset_daily(&mut self) {
        self.daily_loss = 0.0;
        if !self.is_circuit_broken || self.daily_loss < self.daily_loss_limit {
            // Снимаем circuit breaker только если drawdown восстановился
            let dd_pct = if self.peak_equity > 0.0 {
                ((self.peak_equity - self.equity) / self.peak_equity) * 100.0
            } else { 0.0 };
            if dd_pct < self.circuit_breaker_pct * 0.5 {
                self.is_circuit_broken = false;
                println!("  🟢 [PAPER] Circuit breaker RESET — drawdown recovered to {dd_pct:.1}%");
            }
        }
    }

    /// Получить полную статистику сессии.
    pub fn stats(&self) -> PaperStats {
        let wins: Vec<f64> = self.pnl_history.iter().filter(|p| **p > 0.0).copied().collect();
        let losses: Vec<f64> = self.pnl_history.iter().filter(|p| **p < 0.0).copied().collect();

        let total_trades = self.pnl_history.len() as u64;
        let win_count = wins.len() as u64;
        let loss_count = losses.len() as u64;
        let win_rate = if total_trades > 0 { win_count as f64 / total_trades as f64 } else { 0.0 };

        let total_pnl: f64 = self.pnl_history.iter().sum();
        let avg_win = if !wins.is_empty() { wins.iter().sum::<f64>() / wins.len() as f64 } else { 0.0 };
        let avg_loss = if !losses.is_empty() { losses.iter().sum::<f64>() / losses.len() as f64 } else { 0.0 };

        let gross_profit: f64 = wins.iter().sum();
        let gross_loss: f64 = losses.iter().map(|l| l.abs()).sum();
        let profit_factor = if gross_loss > 0.0 { gross_profit / gross_loss } else { f64::INFINITY };

        // Sharpe ratio (annualized, assuming daily returns)
        let mean_ret = if total_trades > 0 { total_pnl / total_trades as f64 } else { 0.0 };
        let variance: f64 = self.pnl_history.iter()
            .map(|p| (p - mean_ret).powi(2))
            .sum::<f64>() / (total_trades.max(1) as f64);
        let std_dev = variance.sqrt();
        let sharpe = if std_dev > 0.0 { (mean_ret / std_dev) * (252.0_f64).sqrt() } else { 0.0 };

        let best = self.pnl_history.iter().cloned().fold(0.0_f64, f64::max);
        let worst = self.pnl_history.iter().cloned().fold(0.0_f64, f64::min);

        let last_ts = self.closed_trades.last()
            .and_then(|t| t.closed_at_ms)
            .unwrap_or(self.session_start_ms);

        PaperStats {
            total_trades,
            wins: win_count,
            losses: loss_count,
            win_rate,
            total_pnl,
            max_drawdown: self.max_drawdown,
            peak_equity: self.peak_equity,
            current_equity: self.equity,
            sharpe_ratio: sharpe,
            profit_factor,
            avg_win,
            avg_loss,
            best_trade: best,
            worst_trade: worst,
            session_start_ms: self.session_start_ms,
            session_duration_ms: last_ts - self.session_start_ms,
        }
    }

    /// Красивый отчёт в консоль.
    pub fn print_report(&self) {
        let s = self.stats();
        println!("\n╔══════════════════════════════════════════╗");
        println!("║     📊 PAPER TRADING REPORT              ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ Trades:     {} ({} W / {} L)           ", s.total_trades, s.wins, s.losses);
        println!("║ Win Rate:   {:.1}%                      ", s.win_rate * 100.0);
        println!("║ Total PnL:  ${:.2}                      ", s.total_pnl);
        println!("║ Equity:     ${:.2}                      ", s.current_equity);
        println!("║ Peak:       ${:.2}                      ", s.peak_equity);
        println!("║ Max DD:     ${:.2}                      ", s.max_drawdown);
        println!("║ Sharpe:     {:.2}                       ", s.sharpe_ratio);
        println!("║ PF:         {:.2}                       ", s.profit_factor);
        println!("║ Avg Win:    ${:.2}                      ", s.avg_win);
        println!("║ Avg Loss:   ${:.2}                      ", s.avg_loss);
        println!("║ Best:       ${:.2}                      ", s.best_trade);
        println!("║ Worst:      ${:.2}                      ", s.worst_trade);
        if self.is_circuit_broken {
            println!("║ ⚠️ CIRCUIT BREAKER ACTIVE               ║");
        }
        println!("╚══════════════════════════════════════════╝\n");
    }

    /// Сохранить состояние paper engine на диск.
    pub fn save(&self, path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Загрузить состояние paper engine.
    pub fn load(path: &str) -> Option<Self> {
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(symbol: &str, price: f64, ts: i64) -> MarketTick {
        MarketTick {
            symbol: symbol.to_string(),
            price,
            high: price * 1.001,
            low: price * 0.999,
            volume: 1000.0,
            timestamp_ms: ts,
        }
    }

    #[test]
    fn test_open_and_close_profitable() {
        let mut engine = PaperEngine::new(1000.0);
        let id = engine.open_position("BTC", Side::Long, 100.0, 1.0, None, None, 1000).unwrap();
        assert_eq!(id, 1);
        assert_eq!(engine.open_positions.len(), 1);

        let pnl = engine.close_position("BTC", 110.0, 2000).unwrap();
        assert!((pnl - 10.0).abs() < 1e-10);
        assert_eq!(engine.balance, 1010.0);
        assert!(engine.open_positions.is_empty());
    }

    #[test]
    fn test_open_and_close_loss() {
        let mut engine = PaperEngine::new(1000.0);
        engine.open_position("ETH", Side::Long, 100.0, 1.0, None, None, 1000).unwrap();
        let pnl = engine.close_position("ETH", 90.0, 2000).unwrap();
        assert!((pnl - (-10.0)).abs() < 1e-10);
        assert_eq!(engine.balance, 990.0);
    }

    #[test]
    fn test_short_position() {
        let mut engine = PaperEngine::new(1000.0);
        engine.open_position("BTC", Side::Short, 100.0, 1.0, None, None, 1000).unwrap();
        let pnl = engine.close_position("BTC", 90.0, 2000).unwrap();
        assert!((pnl - 10.0).abs() < 1e-10, "Short profit when price drops");
    }

    #[test]
    fn test_stop_loss_triggered() {
        let mut engine = PaperEngine::new(1000.0);
        engine.open_position("BTC", Side::Long, 100.0, 1.0, Some(95.0), None, 1000).unwrap();

        // Tick with low touching SL
        let t = MarketTick {
            symbol: "BTC".to_string(),
            price: 96.0,
            high: 101.0,
            low: 94.0,  // Below SL of 95
            volume: 500.0,
            timestamp_ms: 2000,
        };
        engine.on_tick(&t);

        assert!(engine.open_positions.is_empty(), "SL should close position");
        assert_eq!(engine.closed_trades.len(), 1);
        assert_eq!(engine.closed_trades[0].status, OrderStatus::StopLoss);
        assert!((engine.closed_trades[0].pnl - (-5.0)).abs() < 1e-10);
    }

    #[test]
    fn test_take_profit_triggered() {
        let mut engine = PaperEngine::new(1000.0);
        engine.open_position("BTC", Side::Long, 100.0, 1.0, None, Some(110.0), 1000).unwrap();

        let t = MarketTick {
            symbol: "BTC".to_string(),
            price: 109.0,
            high: 111.0,  // Above TP of 110
            low: 108.0,
            volume: 500.0,
            timestamp_ms: 2000,
        };
        engine.on_tick(&t);

        assert!(engine.open_positions.is_empty(), "TP should close position");
        assert_eq!(engine.closed_trades[0].status, OrderStatus::TakeProfit);
        assert!((engine.closed_trades[0].pnl - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_circuit_breaker_drawdown() {
        let mut engine = PaperEngine::new(100.0);
        engine.circuit_breaker_pct = 10.0;

        // Lose 11% → circuit breaker
        engine.open_position("BTC", Side::Long, 100.0, 0.4, None, None, 1000).unwrap();
        engine.close_position("BTC", 72.5, 2000); // -$11 on 0.4 qty = (72.5-100)*0.4 = -11

        assert!(engine.is_circuit_broken, "Should trigger circuit breaker at -11%");

        // Try to open new position → should fail
        let result = engine.open_position("ETH", Side::Long, 50.0, 0.1, None, None, 3000);
        assert!(result.is_err(), "Should reject new orders when circuit broken");
    }

    #[test]
    fn test_daily_loss_limit() {
        let mut engine = PaperEngine::new(1000.0);
        engine.daily_loss_limit = 20.0;

        engine.open_position("A", Side::Long, 100.0, 1.0, None, None, 1000).unwrap();
        engine.close_position("A", 85.0, 2000); // -$15

        engine.open_position("B", Side::Long, 100.0, 1.0, None, None, 3000).unwrap();
        engine.close_position("B", 94.0, 4000); // -$6 → total daily = $21

        assert!(engine.is_circuit_broken, "Daily loss $21 > limit $20");
    }

    #[test]
    fn test_duplicate_position_rejected() {
        let mut engine = PaperEngine::new(1000.0);
        engine.open_position("BTC", Side::Long, 100.0, 0.1, None, None, 1000).unwrap();
        let result = engine.open_position("BTC", Side::Short, 100.0, 0.1, None, None, 2000);
        assert!(result.is_err(), "Should reject duplicate symbol");
    }

    #[test]
    fn test_oversized_position_rejected() {
        let mut engine = PaperEngine::new(100.0);
        // Try 60% of balance → reject
        let result = engine.open_position("BTC", Side::Long, 100.0, 0.61, None, None, 1000);
        assert!(result.is_err(), "Notional $61 > 50% of $100");
    }

    #[test]
    fn test_stats_calculation() {
        let mut engine = PaperEngine::new(1000.0);

        engine.open_position("A", Side::Long, 100.0, 1.0, None, None, 1000).unwrap();
        engine.close_position("A", 110.0, 2000); // +10

        engine.open_position("B", Side::Long, 100.0, 1.0, None, None, 3000).unwrap();
        engine.close_position("B", 95.0, 4000); // -5

        engine.open_position("C", Side::Short, 100.0, 1.0, None, None, 5000).unwrap();
        engine.close_position("C", 92.0, 6000); // +8

        let stats = engine.stats();
        assert_eq!(stats.total_trades, 3);
        assert_eq!(stats.wins, 2);
        assert_eq!(stats.losses, 1);
        assert!((stats.total_pnl - 13.0).abs() < 1e-10);
        assert!(stats.win_rate > 0.6);
        assert!(stats.profit_factor > 1.0);
    }

    #[test]
    fn test_max_drawdown_tracking() {
        let mut engine = PaperEngine::new(1000.0);

        engine.open_position("A", Side::Long, 100.0, 1.0, None, None, 1000).unwrap();
        engine.close_position("A", 120.0, 2000); // +20 → equity 1020

        engine.open_position("B", Side::Long, 100.0, 1.0, None, None, 3000).unwrap();
        engine.close_position("B", 85.0, 4000); // -15 → equity 1005

        assert!((engine.peak_equity - 1020.0).abs() < 1e-10);
        assert!((engine.max_drawdown - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let mut engine = PaperEngine::new(500.0);
        engine.open_position("BTC", Side::Long, 100.0, 0.1, None, None, 1000).unwrap();
        engine.close_position("BTC", 110.0, 2000);

        let tmp = std::env::temp_dir().join("test_paper_engine.json");
        let path = tmp.to_str().unwrap();
        engine.save(path);

        let loaded = PaperEngine::load(path).unwrap();
        assert_eq!(loaded.closed_trades.len(), 1);
        assert!((loaded.balance - engine.balance).abs() < 1e-10);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_reset_daily() {
        let mut engine = PaperEngine::new(1000.0);
        engine.daily_loss_limit = 10.0;

        engine.open_position("A", Side::Long, 100.0, 1.0, None, None, 1000).unwrap();
        engine.close_position("A", 88.0, 2000); // -$12

        assert!(engine.is_circuit_broken);

        // New day — but drawdown still high, so CB stays
        engine.reset_daily();
        // DD = peak(1000) - equity(988) = 12 → 1.2% < 5% threshold → RESET
        assert!(!engine.is_circuit_broken, "CB should reset when DD recovers");
        assert_eq!(engine.daily_loss, 0.0);
    }
}
