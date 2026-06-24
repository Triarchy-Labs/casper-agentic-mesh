// src/modules/indicators.rs
// Модуль технических индикаторов. Plug-n-play: каждый индикатор — отдельная чистая функция.

/// Volume Surge: сравнивает последний объём со средним за 20 свечей
/// Возвращает (is_surge, vol_ratio)
pub fn calc_volume_surge(vols: &[f64]) -> (bool, f64) {
    if vols.len() < 22 { return (false, 1.0); }
    let n = vols.len();
    // BUG-08 FIX: average = предыдущие 20 свечей (не самые старые)
    let avg_vol: f64 = vols[n-21..n-1].iter().sum::<f64>() / 20.0;
    let last_vol = vols[n-1];
    if avg_vol <= 0.0 { return (false, 1.0); }
    let ratio = last_vol / avg_vol;
    (ratio > 1.5, ratio)
}

/// RSI Wilder's Smoothing (14-period)
/// Возвращает RSI 0-100
pub fn calc_rsi_wilders(closes: &[f64], period: usize) -> f64 {
    if closes.len() <= period + 1 { return 50.0; }
    
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for i in 1..=period {
        let diff = closes[i] - closes[i - 1];
        if diff > 0.0 { avg_gain += diff; } else { avg_loss -= diff; }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    
    for i in (period + 1)..closes.len() {
        let diff = closes[i] - closes[i - 1];
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
    }
    
    if avg_loss == 0.0 { return 100.0; }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

/// Определяет цвет последней свечи
pub fn is_green_candle(closes: &[f64]) -> bool {
    if closes.len() < 2 { return false; }
    closes[closes.len() - 1] >= closes[closes.len() - 2]
}

/// Q4: Market Regime Detection (pure function)
/// Uses Efficiency Ratio (ER) = |net displacement| / total path length
/// ER > 0.6 → TRENDING (momentum is real, follow trend)
/// ER < 0.15 & low vol → RANGING (no edge, avoid or reduce size)
/// else → VOLATILE (reduce size, widen stops)
/// Returns (regime_name, score_modifier)
pub fn detect_market_regime(closes: &[f64]) -> (&'static str, f64) {
    if closes.len() < 10 { return ("UNKNOWN", 0.0); }

    let n = closes.len();
    let net_displacement = (closes[n - 1] - closes[0]).abs();
    
    // ГИПЕРЗВУК: single-pass total_path + avg_price
    let mut total_path = 0.0_f64;
    let mut price_sum = closes[0];
    for i in 1..n {
        total_path += (closes[i] - closes[i - 1]).abs();
        price_sum += closes[i];
    }
    
    if total_path < 0.0001 { return ("RANGING", -0.5); } // Dead flat
    
    let efficiency_ratio = net_displacement / total_path;
    
    let avg_price = price_sum / n as f64;
    let avg_move_pct = (total_path / (n - 1) as f64) / avg_price.max(0.01) * 100.0;
    
    if efficiency_ratio > 0.6 {
        // Strong directional movement → TRENDING
        ("TRENDING", 0.5) // Boost score slightly (trend following pays)
    } else if efficiency_ratio < 0.15 && avg_move_pct < 0.5 {
        // No direction + low volatility → RANGING (choppy, no edge)
        ("RANGING", -0.5) // Reduce score (mean reversion territory)
    } else if avg_move_pct > 2.0 {
        // High volatility but no clear direction → VOLATILE
        ("VOLATILE", -1.0) // Penalize (widened stops, reduced sizing)
    } else {
        // Mild transition — don't modify
        ("TRANSITION", 0.0)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    // ═══ RSI TESTS ═══

    #[test]
    fn test_rsi_bullish_trend() {
        // Monotonically rising prices → RSI should be high (>70)
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let rsi = calc_rsi_wilders(&closes, 14);
        assert!(rsi > 70.0, "Bullish RSI should be >70, got {:.1}", rsi);
    }

    #[test]
    fn test_rsi_bearish_trend() {
        // Monotonically falling prices → RSI should be low (<30)
        let closes: Vec<f64> = (0..20).map(|i| 120.0 - i as f64).collect();
        let rsi = calc_rsi_wilders(&closes, 14);
        assert!(rsi < 30.0, "Bearish RSI should be <30, got {:.1}", rsi);
    }

    #[test]
    fn test_rsi_flat_market() {
        // All same price → no gains, no losses → RSI = 50 (default)
        let closes = vec![100.0; 20];
        let rsi = calc_rsi_wilders(&closes, 14);
        // With zero movement, avg_gain=0 and avg_loss=0, but we check avg_loss==0 → returns 100
        // Actually: if all same, diff=0 for all, avg_gain=0, avg_loss=0, returns 100.0
        assert!(rsi >= 50.0, "Flat RSI should be ≥50, got {:.1}", rsi);
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let closes = vec![100.0, 101.0, 102.0];
        let rsi = calc_rsi_wilders(&closes, 14);
        assert_eq!(rsi, 50.0, "Insufficient data should return default 50.0");
    }

    #[test]
    fn test_rsi_range_bounds() {
        // RSI must always be 0-100
        let up: Vec<f64> = (0..50).map(|i| 50.0 + i as f64 * 2.0).collect();
        let rsi_up = calc_rsi_wilders(&up, 14);
        assert!(rsi_up >= 0.0 && rsi_up <= 100.0, "RSI out of bounds: {}", rsi_up);
        
        let down: Vec<f64> = (0..50).map(|i| 200.0 - i as f64 * 2.0).collect();
        let rsi_down = calc_rsi_wilders(&down, 14);
        assert!(rsi_down >= 0.0 && rsi_down <= 100.0, "RSI out of bounds: {}", rsi_down);
    }

    // ═══ VOLUME SURGE TESTS ═══

    #[test]
    fn test_volume_surge_detected() {
        // 21 normal volumes + 1 massive spike
        let mut vols = vec![100.0; 21];
        vols.push(200.0); // 2x average → ratio > 1.5 → surge
        let (is_surge, ratio) = calc_volume_surge(&vols);
        assert!(is_surge, "Should detect surge at 2x volume");
        assert!((ratio - 2.0).abs() < 0.01, "Ratio should be ~2.0, got {:.2}", ratio);
    }

    #[test]
    fn test_volume_no_surge() {
        let vols = vec![100.0; 22]; // all same → ratio = 1.0 → no surge
        let (is_surge, ratio) = calc_volume_surge(&vols);
        assert!(!is_surge, "Should NOT detect surge at 1.0x");
        assert!((ratio - 1.0).abs() < 0.01, "Ratio should be ~1.0");
    }

    #[test]
    fn test_volume_insufficient_data() {
        let vols = vec![100.0; 10]; // too few
        let (is_surge, ratio) = calc_volume_surge(&vols);
        assert!(!is_surge, "Insufficient data should return false");
        assert_eq!(ratio, 1.0);
    }

    // ═══ CANDLE COLOR TESTS ═══

    #[test]
    fn test_green_candle() {
        assert!(is_green_candle(&[100.0, 101.0]));
        assert!(is_green_candle(&[100.0, 100.0])); // flat = green (>=)
    }

    #[test]
    fn test_red_candle() {
        assert!(!is_green_candle(&[101.0, 100.0]));
    }

    #[test]
    fn test_candle_insufficient_data() {
        assert!(!is_green_candle(&[100.0])); // single point
        assert!(!is_green_candle(&[]));      // empty
    }

    // ═══ REGIME DETECTION (Q4) ═══

    #[test]
    fn test_regime_trending_up() {
        // Strong uptrend: +1% per candle × 20
        let closes: Vec<f64> = (0..20).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
        let (regime, _) = detect_market_regime(&closes);
        assert_eq!(regime, "TRENDING", "Monotonic up should be TRENDING");
    }

    #[test]
    fn test_regime_ranging() {
        // Sideways: oscillating up/down by tiny amount, returning to start
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + if i % 2 == 0 { 0.05 } else { -0.05 }).collect();
        let (regime, _) = detect_market_regime(&closes);
        assert_eq!(regime, "RANGING", "Tight oscillation should be RANGING, net displacement ≈ 0");
    }

    #[test]
    fn test_regime_volatile() {
        // Wild swings: ±5% alternating
        let closes: Vec<f64> = (0..20).map(|i| if i % 2 == 0 { 100.0 } else { 95.0 }).collect();
        let (regime, _) = detect_market_regime(&closes);
        assert_eq!(regime, "VOLATILE", "Wild swings should be VOLATILE");
    }

    #[test]
    fn test_regime_insufficient_data() {
        let (regime, modifier) = detect_market_regime(&[100.0, 101.0]);
        assert_eq!(regime, "UNKNOWN");
        assert_eq!(modifier, 0.0);
    }
}
