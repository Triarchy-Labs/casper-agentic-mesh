// src/modules/types.rs
// ═══════════════════════════════════════════════════════════════
// SHARED TYPES — Единый дом для типов, используемых across modules
// ═══════════════════════════════════════════════════════════════
// Вынесено из main.rs для устранения crate:: зависимостей.

use chrono::Timelike;

// ═══ ACTIVE POSITION ═══
/// Состояние активной торговой позиции
#[derive(Clone, Debug)]
#[allow(dead_code)] // symbol read via Debug trait + crash recovery serialization
pub struct ActivePosition {
    pub symbol: String,
    pub side: String,
    pub amount: f64,
    pub buy_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub owner_timeframe: String,
    pub last_pushed_sl: f64,
    pub entry_time_ms: i64,       // DNA V2: timestamp at position open
    pub unstuck_stage1_done: bool, // UNSTUCK: stage 1 (30%) completed
    pub unstuck_stage1_time: i64,  // UNSTUCK: timestamp of stage 1 execution
    pub pending_reentry_price: Option<f64>, // AUTO RE-ENTRY: target price after trim
    pub pending_reentry_qty: Option<f64>,   // AUTO RE-ENTRY: quantity to re-enter
}

// ═══ SESSION MANAGEMENT ═══
/// Определяет текущую торговую сессию по UTC часу
pub fn get_current_session() -> &'static str {
    let hour = chrono::Utc::now().hour();
    match hour {
        23 | 0..=6  => "ASIA",
        7..=14      => "EUROPE",
        15..=22     => "US",
        _           => "US",
    }
}

/// Время начала следующей сессии в UTC timestamp
#[allow(dead_code)] // FORENSIC-16: kept for future DMS session-boundary use
pub fn get_next_session_start_utc() -> i64 {
    let now = chrono::Utc::now();
    let hour = now.hour();
    let today = now.date_naive();
    let next_hour = match hour {
        23 | 0..=6  => 7,
        7..=14      => 15,
        15..=22     => 23,
        _           => 23,
    };
    use chrono::NaiveTime;
    let next_time = if next_hour > hour {
        today.and_time(NaiveTime::from_hms_opt(next_hour, 0, 0).expect("valid HMS"))
    } else {
        (today + chrono::Duration::days(1)).and_time(NaiveTime::from_hms_opt(next_hour, 0, 0).expect("valid HMS"))
    };
    next_time.and_utc().timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_returns_valid_name() {
        let session = get_current_session();
        assert!(
            session == "ASIA" || session == "EUROPE" || session == "US",
            "Session must be ASIA/EUROPE/US, got: {}", session
        );
    }

    #[test]
    fn test_next_session_in_future() {
        let next = get_next_session_start_utc();
        let now = chrono::Utc::now().timestamp();
        assert!(next > now, "Next session start must be in the future");
    }

    #[test]
    fn test_active_position_default_state() {
        let pos = ActivePosition {
            symbol: "BTCUSDT".to_string(),
            side: "Buy".to_string(),
            amount: 0.01,
            buy_price: 95000.0,
            highest_price: 95500.0,
            lowest_price: 94800.0,
            owner_timeframe: "3".to_string(),
            last_pushed_sl: 94000.0,
            entry_time_ms: 1700000000000,
            unstuck_stage1_done: false,
            unstuck_stage1_time: 0,
            pending_reentry_price: None,
            pending_reentry_qty: None,
        };
        assert_eq!(pos.symbol, "BTCUSDT");
        assert!(!pos.unstuck_stage1_done);
        assert_eq!(pos.unstuck_stage1_time, 0);
        assert!(pos.pending_reentry_price.is_none());
    }
}
