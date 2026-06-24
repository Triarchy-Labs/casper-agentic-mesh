/// Technical Analysis Indicators — lightweight Rust-native implementations.
///
/// ПОРТИРОВАНО ИЗ: VectorTA (280+ indicators, Apache-2.0)
/// АВТОР ОРИГИНАЛА: VectorAlpha-dev
///
/// Срезаны CORE формулы без SIMD/Kernel/Candles обёрток.
/// Каждый индикатор принимает &[f64] → возвращает Vec<f64>.
///
/// Содержит 8 ключевых индикаторов для Score Judge:
///   1. RSI (Relative Strength Index) — перекупленность/перепроданность
///   2. EMA (Exponential Moving Average) — сглаженный тренд
///   3. MACD (Moving Average Convergence Divergence) — momentum
///   4. ATR (Average True Range) — волатильность
///   5. Bollinger Bands — ценовые каналы
///   6. ADX (Average Directional Index) — сила тренда
///   7. Stochastic RSI — нормализованный RSI
///   8. OBV (On-Balance Volume) — объёмное давление
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// 1. RSI (порт: VectorTA/indicators/rsi.rs:327-413)
// ═══════════════════════════════════════════════════════════════

/// Relative Strength Index using Wilder's smoothing (EWMA).
/// Returns values 0-100 where >70 = overbought, <30 = oversold.
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    let len = data.len();
    if len < period + 1 || period == 0 {
        return vec![f64::NAN; len];
    }

    let mut out = vec![f64::NAN; len];
    let inv_p = 1.0 / period as f64;
    let beta = 1.0 - inv_p;

    // Initial SMA of gains/losses
    let mut avg_gain = 0.0_f64;
    let mut avg_loss = 0.0_f64;
    for i in 1..=period {
        let delta = data[i] - data[i - 1];
        if delta > 0.0 { avg_gain += delta; }
        else if delta < 0.0 { avg_loss -= delta; }
    }
    avg_gain *= inv_p;
    avg_loss *= inv_p;

    let denom = avg_gain + avg_loss;
    out[period] = if denom == 0.0 { 50.0 } else { 100.0 * avg_gain / denom };

    // EWMA smoothing (порт: rsi.rs:372-413)
    for j in (period + 1)..len {
        let d = data[j] - data[j - 1];
        let g = if d > 0.0 { d } else { 0.0 };
        let l = if d < 0.0 { -d } else { 0.0 };
        avg_gain = avg_gain.mul_add(beta, inv_p * g);
        avg_loss = avg_loss.mul_add(beta, inv_p * l);
        let denom = avg_gain + avg_loss;
        out[j] = if denom == 0.0 { 50.0 } else { 100.0 * avg_gain / denom };
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 2. EMA (core building block)
// ═══════════════════════════════════════════════════════════════

/// Exponential Moving Average.
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    let len = data.len();
    if len < period || period == 0 {
        return vec![f64::NAN; len];
    }

    let mut out = vec![f64::NAN; len];
    let alpha = 2.0 / (period as f64 + 1.0);

    // SMA seed
    let sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = sma;

    let mut prev = sma;
    for i in period..len {
        prev = data[i] * alpha + prev * (1.0 - alpha);
        out[i] = prev;
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 3. MACD (порт: VectorTA/indicators/macd.rs core)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct MacdOutput {
    pub macd_line: Vec<f64>,     // fast_ema - slow_ema
    pub signal_line: Vec<f64>,   // EMA of macd_line
    pub histogram: Vec<f64>,     // macd - signal
}

/// MACD (12, 26, 9 default).
/// Порт: VectorTA macd.rs core scalar path.
pub fn macd(data: &[f64], fast: usize, slow: usize, signal_period: usize) -> MacdOutput {
    let fast_ema = ema(data, fast);
    let slow_ema = ema(data, slow);
    let len = data.len();

    let mut macd_line = vec![f64::NAN; len];
    for i in 0..len {
        if !fast_ema[i].is_nan() && !slow_ema[i].is_nan() {
            macd_line[i] = fast_ema[i] - slow_ema[i];
        }
    }

    // Signal = EMA of MACD line (skip NaNs)
    let valid_start = macd_line.iter().position(|x| !x.is_nan()).unwrap_or(len);
    let macd_valid: Vec<f64> = macd_line[valid_start..].to_vec();
    let signal_raw = ema(&macd_valid, signal_period);

    let mut signal_line = vec![f64::NAN; len];
    for (i, &v) in signal_raw.iter().enumerate() {
        signal_line[valid_start + i] = v;
    }

    let mut histogram = vec![f64::NAN; len];
    for i in 0..len {
        if !macd_line[i].is_nan() && !signal_line[i].is_nan() {
            histogram[i] = macd_line[i] - signal_line[i];
        }
    }

    MacdOutput { macd_line, signal_line, histogram }
}

// ═══════════════════════════════════════════════════════════════
// 4. ATR (порт: VectorTA/indicators/atr.rs core)
// ═══════════════════════════════════════════════════════════════

/// Average True Range from High/Low/Close arrays.
/// true_range = max(H-L, |H-prevC|, |L-prevC|)
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let len = high.len().min(low.len()).min(close.len());
    if len < 2 || period == 0 {
        return vec![f64::NAN; len];
    }

    // True Range
    let mut tr = vec![0.0_f64; len];
    tr[0] = high[0] - low[0];
    for i in 1..len {
        let hl = high[i] - low[i];
        let hc = (high[i] - close[i - 1]).abs();
        let lc = (low[i] - close[i - 1]).abs();
        tr[i] = hl.max(hc).max(lc);
    }

    // Wilder's smoothing
    let mut out = vec![f64::NAN; len];
    if period > len { return out; }

    let first_atr: f64 = tr[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = first_atr;

    let mut prev = first_atr;
    for i in period..len {
        prev = (prev * (period as f64 - 1.0) + tr[i]) / period as f64;
        out[i] = prev;
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 5. Bollinger Bands (порт: VectorTA/indicators/bollinger_bands.rs)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct BollingerOutput {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,  // SMA
    pub lower: Vec<f64>,
    pub width: Vec<f64>,   // (upper - lower) / middle
}

/// Bollinger Bands (20, 2.0 default).
pub fn bollinger_bands(data: &[f64], period: usize, std_dev: f64) -> BollingerOutput {
    let len = data.len();
    let mut upper = vec![f64::NAN; len];
    let mut middle = vec![f64::NAN; len];
    let mut lower = vec![f64::NAN; len];
    let mut width = vec![f64::NAN; len];

    if period == 0 || len < period {
        return BollingerOutput { upper, middle, lower, width };
    }

    for i in (period - 1)..len {
        let window = &data[(i + 1 - period)..=i];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let sd = variance.sqrt();

        middle[i] = mean;
        upper[i] = mean + std_dev * sd;
        lower[i] = mean - std_dev * sd;
        width[i] = if mean != 0.0 { (upper[i] - lower[i]) / mean } else { 0.0 };
    }

    BollingerOutput { upper, middle, lower, width }
}

// ═══════════════════════════════════════════════════════════════
// 6. ADX (порт: VectorTA/indicators/adx.rs core)
// ═══════════════════════════════════════════════════════════════

/// Average Directional Index — trend strength 0-100.
/// >25 = trending, <20 = ranging.
pub fn adx(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let len = high.len().min(low.len()).min(close.len());
    if len < period * 2 || period == 0 {
        return vec![f64::NAN; len];
    }

    // +DM, -DM
    let mut pdm = vec![0.0_f64; len];
    let mut ndm = vec![0.0_f64; len];
    for i in 1..len {
        let up = high[i] - high[i - 1];
        let dn = low[i - 1] - low[i];
        if up > dn && up > 0.0 { pdm[i] = up; }
        if dn > up && dn > 0.0 { ndm[i] = dn; }
    }

    let atr_vals = atr(high, low, close, period);

    // Smoothed +DM, -DM
    let mut s_pdm = vec![0.0_f64; len];
    let mut s_ndm = vec![0.0_f64; len];

    let sum_pdm: f64 = pdm[1..=period].iter().sum();
    let sum_ndm: f64 = ndm[1..=period].iter().sum();
    s_pdm[period] = sum_pdm;
    s_ndm[period] = sum_ndm;

    for i in (period + 1)..len {
        s_pdm[i] = s_pdm[i - 1] - s_pdm[i - 1] / period as f64 + pdm[i];
        s_ndm[i] = s_ndm[i - 1] - s_ndm[i - 1] / period as f64 + ndm[i];
    }

    // +DI, -DI, DX
    let mut dx = vec![f64::NAN; len];
    for i in period..len {
        if let Some(a) = atr_vals.get(i) {
            if *a > 0.0 && !a.is_nan() {
                let pdi = 100.0 * s_pdm[i] / a;
                let ndi = 100.0 * s_ndm[i] / a;
                let sum = pdi + ndi;
                dx[i] = if sum > 0.0 { 100.0 * (pdi - ndi).abs() / sum } else { 0.0 };
            }
        }
    }

    // ADX = Wilder's smoothed DX
    let mut out = vec![f64::NAN; len];
    let adx_start = period * 2 - 1;
    if adx_start >= len { return out; }

    let first_adx: f64 = dx[period..adx_start + 1]
        .iter()
        .filter(|x| !x.is_nan())
        .sum::<f64>() / period as f64;
    out[adx_start] = first_adx;

    let mut prev = first_adx;
    for i in (adx_start + 1)..len {
        if !dx[i].is_nan() {
            prev = (prev * (period as f64 - 1.0) + dx[i]) / period as f64;
            out[i] = prev;
        }
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 7. Stochastic RSI (порт: VectorTA/indicators/srsi.rs)
// ═══════════════════════════════════════════════════════════════

/// Stochastic RSI — normalizes RSI into 0-100 range.
/// period: RSI period, stoch_period: stochastic lookback
pub fn stochastic_rsi(data: &[f64], rsi_period: usize, stoch_period: usize) -> Vec<f64> {
    let rsi_vals = rsi(data, rsi_period);
    let len = rsi_vals.len();
    let mut out = vec![f64::NAN; len];

    for i in (rsi_period + stoch_period - 1)..len {
        let window = &rsi_vals[(i + 1 - stoch_period)..=i];
        let valid: Vec<f64> = window.iter().filter(|x| !x.is_nan()).copied().collect();
        if valid.len() < stoch_period { continue; }

        let min = valid.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        out[i] = if range > 0.0 {
            ((rsi_vals[i] - min) / range * 100.0).clamp(0.0, 100.0)
        } else {
            50.0
        };
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 8. OBV — On-Balance Volume (порт: VectorTA/indicators/obv.rs)
// ═══════════════════════════════════════════════════════════════

/// On-Balance Volume — cumulative volume flow.
/// close: prices, volume: trade volumes
pub fn obv(close: &[f64], volume: &[f64]) -> Vec<f64> {
    let len = close.len().min(volume.len());
    if len == 0 { return vec![]; }

    let mut out = vec![0.0_f64; len];
    out[0] = volume[0];
    for i in 1..len {
        if close[i] > close[i - 1] {
            out[i] = out[i - 1] + volume[i];
        } else if close[i] < close[i - 1] {
            out[i] = out[i - 1] - volume[i];
        } else {
            out[i] = out[i - 1];
        }
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// Convenience: get latest value
// ═══════════════════════════════════════════════════════════════

/// Extract last non-NaN value from indicator output.
pub fn last_valid(vals: &[f64]) -> Option<f64> {
    vals.iter().rev().find(|x| !x.is_nan()).copied()
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prices() -> Vec<f64> {
        vec![
            44.0, 44.3, 44.1, 43.6, 44.3, 44.8, 45.1, 45.4, 45.0, 44.6,
            44.5, 44.7, 44.2, 44.1, 44.4, 44.6, 44.8, 44.5, 44.2, 44.0,
            44.3, 44.5, 44.7, 44.9, 45.1, 45.3, 45.2, 44.9, 44.7, 44.5,
        ]
    }

    #[test]
    fn test_rsi_range() {
        let prices = sample_prices();
        let vals = rsi(&prices, 14);
        for &v in vals.iter().skip(14) {
            assert!(!v.is_nan(), "RSI should not be NaN after warmup");
            assert!(v >= 0.0 && v <= 100.0, "RSI must be 0-100, got {}", v);
        }
    }

    #[test]
    fn test_rsi_warmup() {
        let prices = sample_prices();
        let vals = rsi(&prices, 14);
        for &v in &vals[..14] {
            assert!(v.is_nan(), "RSI should be NaN during warmup");
        }
    }

    #[test]
    fn test_rsi_constant_price() {
        let flat = vec![100.0; 30];
        let vals = rsi(&flat, 14);
        for &v in vals.iter().skip(14) {
            assert!((v - 50.0).abs() < 0.01, "Flat price → RSI=50, got {}", v);
        }
    }

    #[test]
    fn test_ema_converges() {
        let prices = sample_prices();
        let vals = ema(&prices, 10);
        assert!(vals[9].is_finite(), "EMA should start at period-1");
        assert!(vals.last().unwrap().is_finite());
    }

    #[test]
    fn test_macd_structure() {
        let prices = sample_prices();
        let m = macd(&prices, 12, 26, 9);
        assert_eq!(m.macd_line.len(), prices.len());
        assert_eq!(m.signal_line.len(), prices.len());
        assert_eq!(m.histogram.len(), prices.len());
    }

    #[test]
    fn test_atr_positive() {
        let high = vec![45.0, 45.5, 46.0, 45.8, 46.2, 46.5, 46.3, 46.0, 45.5, 45.0, 45.2, 45.5, 45.8, 46.0, 46.3];
        let low  = vec![44.0, 44.5, 45.0, 44.8, 45.2, 45.5, 45.3, 45.0, 44.5, 44.0, 44.2, 44.5, 44.8, 45.0, 45.3];
        let close= vec![44.5, 45.0, 45.5, 45.3, 45.7, 46.0, 45.8, 45.5, 45.0, 44.5, 44.7, 45.0, 45.3, 45.5, 45.8];
        let vals = atr(&high, &low, &close, 5);
        for &v in vals.iter().skip(4) {
            assert!(v > 0.0 && v.is_finite(), "ATR must be positive, got {}", v);
        }
    }

    #[test]
    fn test_bollinger_bands_order() {
        let prices = sample_prices();
        let bb = bollinger_bands(&prices, 10, 2.0);
        for i in 9..prices.len() {
            assert!(bb.lower[i] <= bb.middle[i], "lower <= middle");
            assert!(bb.middle[i] <= bb.upper[i], "middle <= upper");
        }
    }

    #[test]
    fn test_adx_range() {
        let n = 50;
        let high: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.5).sin() * 2.0).collect();
        let low: Vec<f64> = high.iter().map(|h| h - 1.0).collect();
        let close: Vec<f64> = high.iter().zip(low.iter()).map(|(h, l)| (h + l) / 2.0).collect();
        let vals = adx(&high, &low, &close, 14);
        for &v in vals.iter().filter(|x| !x.is_nan()) {
            assert!(v >= 0.0 && v <= 100.0, "ADX 0-100, got {}", v);
        }
    }

    #[test]
    fn test_stochastic_rsi_range() {
        let prices = sample_prices();
        let vals = stochastic_rsi(&prices, 14, 14);
        for &v in vals.iter().filter(|x| !x.is_nan()) {
            assert!(v >= 0.0 && v <= 100.0, "StochRSI 0-100, got {}", v);
        }
    }

    #[test]
    fn test_obv_direction() {
        let close = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let vol   = vec![100.0, 150.0, 200.0, 180.0, 120.0];
        let vals = obv(&close, &vol);
        assert_eq!(vals.len(), 5);
        assert!(vals[2] > vals[0], "Rising prices → OBV increases");
    }

    #[test]
    fn test_last_valid_fn() {
        let vals = vec![f64::NAN, f64::NAN, 42.0, 55.0, f64::NAN];
        assert_eq!(last_valid(&vals), Some(55.0));
    }

    #[test]
    fn test_empty_input() {
        assert!(rsi(&[], 14).is_empty());
        assert!(ema(&[], 10).is_empty());
        assert!(obv(&[], &[]).is_empty());
    }
}
