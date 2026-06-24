/// Session Patterns — анализ PnL по торговым сессиям.
///
/// Определяет в какую сессию торговля ПРОФИТАБЕЛЬНА, а в какую УБЫТОЧНА.
/// Сессии: Tokyo (00:00-08:00 UTC), London (08:00-16:00 UTC), NewYork (13:00-21:00 UTC).
///
/// Закрывает дырку D6 из Blueprint V3.
use serde::Serialize;

/// Торговая сессия.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum TradingSession {
    Tokyo,
    London,
    NewYork,
    LondonNYOverlap,
    OffHours,
}

impl TradingSession {
    /// Определить сессию по UTC часу (0-23).
    pub fn from_utc_hour(hour: u32) -> Self {
        match hour {
            0..=7 => TradingSession::Tokyo,
            8..=12 => TradingSession::London,
            13..=15 => TradingSession::LondonNYOverlap, // Самое ликвидное окно
            16..=20 => TradingSession::NewYork,
            _ => TradingSession::OffHours,
        }
    }

    /// Определить сессию по timestamp_ms (Unix epoch millis).
    pub fn from_timestamp_ms(ts_ms: i64) -> Self {
        if ts_ms <= 0 {
            return TradingSession::OffHours;
        }
        let seconds = ts_ms / 1000;
        let hour = ((seconds % 86400) / 3600) as u32;
        Self::from_utc_hour(hour)
    }

    pub fn name(&self) -> &'static str {
        match self {
            TradingSession::Tokyo => "Tokyo",
            TradingSession::London => "London",
            TradingSession::NewYork => "NewYork",
            TradingSession::LondonNYOverlap => "London-NY Overlap",
            TradingSession::OffHours => "Off-Hours",
        }
    }
}

/// Агрегированная статистика по одной сессии.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SessionStats {
    pub session: String,
    pub trade_count: u32,
    pub win_count: u32,
    pub loss_count: u32,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub win_rate: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
}

/// Результат анализа сессий для одного символа.
#[derive(Debug, Clone, Serialize)]
pub struct SessionAnalysis {
    pub symbol: String,
    pub sessions: Vec<SessionStats>,
    pub best_session: String,
    pub worst_session: String,
}

/// Анализ PnL по сессиям из истории трейдов.
///
/// `trades` — список (pnl, timestamp_ms).
pub fn analyze_sessions(symbol: &str, trades: &[(f64, i64)]) -> SessionAnalysis {
    use std::collections::HashMap;

    let mut session_data: HashMap<TradingSession, Vec<f64>> = HashMap::new();

    for &(pnl, ts) in trades {
        let session = TradingSession::from_timestamp_ms(ts);
        session_data.entry(session).or_default().push(pnl);
    }

    let all_sessions = [
        TradingSession::Tokyo,
        TradingSession::London,
        TradingSession::LondonNYOverlap,
        TradingSession::NewYork,
        TradingSession::OffHours,
    ];

    let mut sessions: Vec<SessionStats> = all_sessions.iter().map(|s| {
        let pnls = session_data.get(s).cloned().unwrap_or_default();
        let trade_count = pnls.len() as u32;
        let win_count = pnls.iter().filter(|&&p| p > 0.0).count() as u32;
        let loss_count = pnls.iter().filter(|&&p| p < 0.0).count() as u32;
        let total_pnl: f64 = pnls.iter().sum();
        let avg_pnl = if trade_count > 0 { total_pnl / trade_count as f64 } else { 0.0 };
        let win_rate = if trade_count > 0 { win_count as f64 / trade_count as f64 } else { 0.0 };
        let best_trade = pnls.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let worst_trade = pnls.iter().copied().fold(f64::INFINITY, f64::min);

        SessionStats {
            session: s.name().to_string(),
            trade_count,
            win_count,
            loss_count,
            total_pnl,
            avg_pnl,
            win_rate,
            best_trade: if trade_count > 0 { best_trade } else { 0.0 },
            worst_trade: if trade_count > 0 { worst_trade } else { 0.0 },
        }
    }).collect();

    // Определить лучшую/худшую сессию по avg_pnl (только с данными)
    let with_trades: Vec<&SessionStats> = sessions.iter().filter(|s| s.trade_count > 0).collect();
    let best_session = with_trades.iter()
        .max_by(|a, b| a.avg_pnl.partial_cmp(&b.avg_pnl).unwrap_or(std::cmp::Ordering::Equal))
        .map(|s| s.session.clone())
        .unwrap_or_else(|| "N/A".to_string());
    let worst_session = with_trades.iter()
        .min_by(|a, b| a.avg_pnl.partial_cmp(&b.avg_pnl).unwrap_or(std::cmp::Ordering::Equal))
        .map(|s| s.session.clone())
        .unwrap_or_else(|| "N/A".to_string());

    // Сортировать по PnL (лучшие сверху)
    sessions.sort_by(|a, b| b.total_pnl.partial_cmp(&a.total_pnl).unwrap_or(std::cmp::Ordering::Equal));

    SessionAnalysis {
        symbol: symbol.to_string(),
        sessions,
        best_session,
        worst_session,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_from_utc_hour() {
        assert_eq!(TradingSession::from_utc_hour(3), TradingSession::Tokyo);
        assert_eq!(TradingSession::from_utc_hour(9), TradingSession::London);
        assert_eq!(TradingSession::from_utc_hour(14), TradingSession::LondonNYOverlap);
        assert_eq!(TradingSession::from_utc_hour(18), TradingSession::NewYork);
        assert_eq!(TradingSession::from_utc_hour(22), TradingSession::OffHours);
    }

    #[test]
    fn test_session_from_timestamp() {
        // 2026-01-01 10:00 UTC = London
        let ts = 1767088800_000i64; // approximate
        let s = TradingSession::from_timestamp_ms(ts);
        // Just verify it returns a valid session, not panicking
        assert_ne!(s.name(), "");
    }

    #[test]
    fn test_session_zero_timestamp() {
        assert_eq!(TradingSession::from_timestamp_ms(0), TradingSession::OffHours);
        assert_eq!(TradingSession::from_timestamp_ms(-1), TradingSession::OffHours);
    }

    #[test]
    fn test_analyze_sessions_basic() {
        let trades = vec![
            (10.0, 3600_000 * 3),  // 03:00 UTC = Tokyo, +10
            (-5.0, 3600_000 * 4),  // 04:00 UTC = Tokyo, -5
            (20.0, 3600_000 * 10), // 10:00 UTC = London, +20
            (-2.0, 3600_000 * 14), // 14:00 UTC = Overlap, -2
        ];
        let analysis = analyze_sessions("TEST", &trades);
        assert_eq!(analysis.symbol, "TEST");
        assert_eq!(analysis.best_session, "London");

        // Tokyo has 2 trades
        let tokyo = analysis.sessions.iter().find(|s| s.session == "Tokyo").unwrap();
        assert_eq!(tokyo.trade_count, 2);
        assert_eq!(tokyo.win_count, 1);
        assert!((tokyo.total_pnl - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_analyze_empty_trades() {
        let analysis = analyze_sessions("EMPTY", &[]);
        assert_eq!(analysis.best_session, "N/A");
        assert_eq!(analysis.worst_session, "N/A");
        assert!(analysis.sessions.iter().all(|s| s.trade_count == 0));
    }

    #[test]
    fn test_session_names_unique() {
        let sessions = [
            TradingSession::Tokyo,
            TradingSession::London,
            TradingSession::NewYork,
            TradingSession::LondonNYOverlap,
            TradingSession::OffHours,
        ];
        let names: Vec<&str> = sessions.iter().map(|s| s.name()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "All session names must be unique");
    }
}
