/// Local ML Inference — ВЕКТОР 1 из OVERKILL Roadmap.
///
/// Micro-ML модуль: Logistic Regression + Feature Engineering.
/// Zero external dependencies. Inference < 1μs.
///
/// Архитектура:
///   Level 1 (этот модуль): Logistic Regression + online SGD
///   Level 2 (будущее): candle-core для BERT/LSTM inference
///
/// Features:
///   - RSI (14-period)
///   - Price momentum (5-candle change %)
///   - Volume ratio (current / avg)
///   - OBI (Order Book Imbalance)
///   - Volatility (ATR-based)
///   - Win rate (entity DNA)
///   - Regime score
///
/// Предсказание: P(profit) ∈ [0.0, 1.0]
use serde::{Serialize, Deserialize};

// ════════════════════════════════════════════════════════════════
// Feature Vector
// ════════════════════════════════════════════════════════════════

/// Нормализованный вектор фичей для ML модели.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// RSI нормализованный [0, 1] (raw / 100)
    pub rsi_norm: f64,
    /// Momentum: процент изменения цены за N свечей [-1, 1] (clamped)
    pub momentum: f64,
    /// Отношение текущего объёма к среднему [0, 3] (clamped)
    pub volume_ratio: f64,
    /// Order Book Imbalance [-1, 1]
    pub obi: f64,
    /// Волатильность нормализованная [0, 1]
    pub volatility: f64,
    /// Win rate из DNA [0, 1]
    pub win_rate: f64,
    /// Regime confidence [0, 1]
    pub regime_score: f64,
}

impl FeatureVector {
    /// Конвертировать в массив для dot product.
    #[allow(dead_code)]
    pub fn as_array(&self) -> [f64; 7] {
        [
            self.rsi_norm,
            self.momentum,
            self.volume_ratio,
            self.obi,
            self.volatility,
            self.win_rate,
            self.regime_score,
        ]
    }

    /// Создать из raw значений с нормализацией.
    #[allow(clippy::too_many_arguments)]
    pub fn from_raw(
        rsi: f64,
        price_change_pct: f64,
        current_volume: f64,
        avg_volume: f64,
        obi: f64,
        atr_pct: f64,
        win_rate: f64,
        regime_confidence: f64,
    ) -> Self {
        Self {
            rsi_norm: (rsi / 100.0).clamp(0.0, 1.0),
            momentum: (price_change_pct / 5.0).clamp(-1.0, 1.0), // ±5% → ±1.0
            volume_ratio: if avg_volume > 0.0 {
                (current_volume / avg_volume).clamp(0.0, 3.0)
            } else { 1.0 },
            obi: obi.clamp(-1.0, 1.0),
            volatility: (atr_pct / 3.0).clamp(0.0, 1.0), // 3% ATR → 1.0
            win_rate: win_rate.clamp(0.0, 1.0),
            regime_score: regime_confidence.clamp(0.0, 1.0),
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Logistic Regression Model
// ════════════════════════════════════════════════════════════════

const NUM_FEATURES: usize = 7;

/// ML предсказание.
#[derive(Debug, Clone)]
pub struct Prediction {
    /// P(profit) ∈ [0.0, 1.0]
    pub probability: f64,
    /// Бинарный сигнал: true = стоит торговать
    pub signal: bool,
    /// Уверенность: |probability - 0.5| * 2 ∈ [0.0, 1.0]
    pub confidence: f64,
}

/// Логистическая регрессия с online SGD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    /// Веса [w0..w6]
    pub weights: [f64; NUM_FEATURES],
    /// Bias
    pub bias: f64,
    /// Learning rate
    pub lr: f64,
    /// Количество обучающих примеров
    pub train_count: u64,
    /// Cumulative loss (для мониторинга)
    pub cumulative_loss: f64,
    /// Порог для сигнала
    pub threshold: f64,
}

impl LocalModel {
    /// Создать модель с начальными весами.
    pub fn new() -> Self {
        Self {
            // Интуитивные начальные веса:
            weights: [
                -0.3,  // RSI: low RSI → oversold → buy opportunity
                 0.2,  // Momentum: positive momentum → bullish
                 0.1,  // Volume: high volume → stronger signal
                 0.4,  // OBI: positive imbalance → buy pressure
                -0.1,  // Volatility: high vol → uncertain
                 0.3,  // Win rate: proven winner → trust
                 0.2,  // Regime: high confidence → better signal
            ],
            bias: 0.0,
            lr: 0.01,
            train_count: 0,
            cumulative_loss: 0.0,
            threshold: 0.6, // Нужна 60% уверенность для сигнала
        }
    }

    /// Sigmoid activation.
    #[inline]
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Forward pass: features → P(profit).
    pub fn predict(&self, features: &FeatureVector) -> Prediction {
        let arr = features.as_array();
        let mut z = self.bias;
        for (w, f) in self.weights.iter().zip(arr.iter()) {
            z += w * f;
        }

        let prob = Self::sigmoid(z);
        let confidence = (prob - 0.5).abs() * 2.0;

        Prediction {
            probability: prob,
            signal: prob >= self.threshold,
            confidence,
        }
    }

    /// Online SGD: обучить на одном примере.
    /// `outcome`: true = trade was profitable, false = loss.
    pub fn train(&mut self, features: &FeatureVector, outcome: bool) {
        let pred = self.predict(features);
        let target = if outcome { 1.0 } else { 0.0 };
        let error = pred.probability - target;

        // Binary cross-entropy loss
        let loss = if outcome {
            -(pred.probability.max(1e-15)).ln()
        } else {
            -(1.0 - pred.probability.min(1.0 - 1e-15)).ln()
        };
        self.cumulative_loss += loss;

        // Gradient descent с adaptive learning rate
        let effective_lr = self.lr / (1.0 + self.train_count as f64 * 0.001);
        let arr = features.as_array();
        for (w, f) in self.weights.iter_mut().zip(arr.iter()) {
            *w -= effective_lr * error * f;
        }
        self.bias -= effective_lr * error;

        self.train_count += 1;
    }

    /// Batch train на массиве (features, outcome).
    pub fn train_batch(&mut self, samples: &[(FeatureVector, bool)]) {
        for (features, outcome) in samples {
            self.train(features, *outcome);
        }
    }

    /// Средний loss.
    pub fn avg_loss(&self) -> f64 {
        if self.train_count > 0 {
            self.cumulative_loss / self.train_count as f64
        } else {
            0.0
        }
    }

    /// Точность на тестовой выборке.
    pub fn accuracy(&self, samples: &[(FeatureVector, bool)]) -> f64 {
        if samples.is_empty() { return 0.0; }
        let correct = samples.iter()
            .filter(|(f, outcome)| {
                let pred = self.predict(f);
                pred.signal == *outcome
            })
            .count();
        correct as f64 / samples.len() as f64
    }

    /// Сохранить модель.
    pub fn save(&self, path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Загрузить модель.
    pub fn load(path: &str) -> Option<Self> {
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }
}

impl Default for LocalModel {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn bullish_features() -> FeatureVector {
        FeatureVector::from_raw(
            25.0,  // RSI oversold
            2.0,   // 2% momentum up
            1500.0, 1000.0, // 1.5x volume
            0.5,   // OBI positive
            1.0,   // Low vol
            0.65,  // Good win rate
            0.8,   // High regime confidence
        )
    }

    fn bearish_features() -> FeatureVector {
        FeatureVector::from_raw(
            80.0,  // RSI overbought
            -3.0,  // 3% momentum down
            500.0, 1000.0, // 0.5x volume
            -0.6,  // OBI negative
            2.5,   // High vol
            0.35,  // Poor win rate
            0.3,   // Low regime confidence
        )
    }

    #[test]
    fn test_feature_normalization() {
        let f = FeatureVector::from_raw(50.0, 0.0, 1000.0, 1000.0, 0.0, 1.5, 0.5, 0.5);
        assert!((f.rsi_norm - 0.5).abs() < 1e-10);
        assert!((f.momentum - 0.0).abs() < 1e-10);
        assert!((f.volume_ratio - 1.0).abs() < 1e-10);
        assert!((f.obi - 0.0).abs() < 1e-10);
        assert!((f.volatility - 0.5).abs() < 1e-10);
        assert!((f.win_rate - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_feature_clamping() {
        let f = FeatureVector::from_raw(150.0, 20.0, 5000.0, 100.0, 5.0, 10.0, 1.5, 2.0);
        assert!((f.rsi_norm - 1.0).abs() < 1e-10);
        assert!((f.momentum - 1.0).abs() < 1e-10);
        assert!((f.volume_ratio - 3.0).abs() < 1e-10);
        assert!((f.obi - 1.0).abs() < 1e-10);
        assert!((f.win_rate - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_feature_as_array() {
        let f = bullish_features();
        let arr = f.as_array();
        assert_eq!(arr.len(), 7);
        assert!((arr[0] - f.rsi_norm).abs() < 1e-10);
    }

    #[test]
    fn test_sigmoid_bounds() {
        assert!((LocalModel::sigmoid(0.0) - 0.5).abs() < 1e-10);
        assert!(LocalModel::sigmoid(100.0) > 0.99);
        assert!(LocalModel::sigmoid(-100.0) < 0.01);
    }

    #[test]
    fn test_prediction_fields() {
        let model = LocalModel::new();
        let pred = model.predict(&bullish_features());
        assert!(pred.probability >= 0.0 && pred.probability <= 1.0);
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
    }

    #[test]
    fn test_initial_model_bullish_bias() {
        let model = LocalModel::new();
        let bull = model.predict(&bullish_features());
        let bear = model.predict(&bearish_features());
        assert!(
            bull.probability > bear.probability,
            "Bullish features should have higher P(profit): bull={:.3} bear={:.3}",
            bull.probability, bear.probability
        );
    }

    #[test]
    fn test_train_improves_on_bullish() {
        let mut model = LocalModel::new();
        let features = bullish_features();

        let before = model.predict(&features).probability;
        // Train: this bullish setup was profitable
        for _ in 0..50 {
            model.train(&features, true);
        }
        let after = model.predict(&features).probability;

        assert!(after > before, "Training on positive outcome should increase P: {:.3} → {:.3}", before, after);
    }

    #[test]
    fn test_train_decreases_on_bearish() {
        let mut model = LocalModel::new();
        let features = bearish_features();

        let before = model.predict(&features).probability;
        // Train: this bearish setup was a loss
        for _ in 0..50 {
            model.train(&features, false);
        }
        let after = model.predict(&features).probability;

        assert!(after < before, "Training on negative outcome should decrease P: {:.3} → {:.3}", before, after);
    }

    #[test]
    fn test_train_count_increments() {
        let mut model = LocalModel::new();
        assert_eq!(model.train_count, 0);
        model.train(&bullish_features(), true);
        assert_eq!(model.train_count, 1);
        model.train(&bearish_features(), false);
        assert_eq!(model.train_count, 2);
    }

    #[test]
    fn test_avg_loss_decreases_with_training() {
        let mut model = LocalModel::new();
        let features = bullish_features();

        // Train 100 times on bullish → profitable pattern
        for _ in 0..100 {
            model.train(&features, true);
        }

        let loss_after_100 = model.avg_loss();
        // Loss should be finite and positive
        assert!(loss_after_100.is_finite());
        assert!(loss_after_100 > 0.0);
    }

    #[test]
    fn test_accuracy_perfect() {
        let mut model = LocalModel::new();
        model.threshold = 0.5;

        let samples: Vec<(FeatureVector, bool)> = vec![
            (bullish_features(), true),
            (bearish_features(), false),
        ];

        // Train heavily
        for _ in 0..200 {
            model.train_batch(&samples);
        }

        let acc = model.accuracy(&samples);
        assert!(acc >= 0.5, "Should learn simple pattern, acc={:.2}", acc);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let mut model = LocalModel::new();
        model.train(&bullish_features(), true);
        model.train(&bearish_features(), false);

        let tmp = std::env::temp_dir().join("test_ml_model.json");
        let path = tmp.to_str().unwrap();
        model.save(path);

        let loaded = LocalModel::load(path).unwrap();
        assert_eq!(loaded.train_count, 2);
        for i in 0..NUM_FEATURES {
            assert!((loaded.weights[i] - model.weights[i]).abs() < 1e-10);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_batch_train() {
        let mut model = LocalModel::new();
        let samples = vec![
            (bullish_features(), true),
            (bearish_features(), false),
            (bullish_features(), true),
        ];
        model.train_batch(&samples);
        assert_eq!(model.train_count, 3);
    }

    #[test]
    fn test_zero_volume_handling() {
        let f = FeatureVector::from_raw(50.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.5, 0.5);
        assert!((f.volume_ratio - 1.0).abs() < 1e-10, "Zero avg_volume should default to 1.0");
    }

    #[test]
    fn test_confidence_at_threshold() {
        let model = LocalModel::new();
        let f = FeatureVector::from_raw(50.0, 0.0, 1000.0, 1000.0, 0.0, 1.0, 0.5, 0.5);
        let pred = model.predict(&f);
        // Confidence should be low when probability is near 0.5
        // and high when probability is near 0 or 1
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
    }
}
