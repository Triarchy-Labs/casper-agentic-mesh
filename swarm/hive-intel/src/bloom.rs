/// Bloom Filter — O(1) novelty detection.
///
/// "Видел ли я этот контекст раньше?"
/// - false positive возможен (говорит "да" когда не видел)
/// - false negative НЕВОЗМОЖЕН (если говорит "нет" — точно нет)
///
/// Используется для fast-reject в recall: пропускаем полный scoring
/// если контекст точно никогда не встречался.
use serde::{Deserialize, Serialize};

/// Компактный Bloom Filter на фиксированном массиве битов.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilter {
    bits: Vec<u64>,   // Bit array (64-bit words)
    num_bits: usize,
    num_hashes: u32,
    count: usize,
}

impl BloomFilter {
    /// Создаёт Bloom Filter с заданным размером и числом хэш-функций.
    ///
    /// Рекомендация: для 10K элементов, 0.1% false positive:
    /// - bits = 144_000 (18 KB)
    /// - hashes = 10
    pub fn new(num_bits: usize, num_hashes: u32) -> Self {
        let words = num_bits.div_ceil(64);
        Self {
            bits: vec![0u64; words],
            num_bits,
            num_hashes,
            count: 0,
        }
    }

    /// Создаёт оптимальный Bloom Filter для n элементов с заданным false positive rate.
    pub fn optimal(expected_items: usize, fp_rate: f64) -> Self {
        let m = (-(expected_items as f64) * fp_rate.ln() / (2.0_f64.ln().powi(2))).ceil() as usize;
        let k = ((m as f64 / expected_items as f64) * 2.0_f64.ln()).ceil() as u32;
        Self::new(m.max(64), k.clamp(1, 20))
    }

    /// Вставляет элемент (по его хэшу).
    pub fn insert(&mut self, item: u64) {
        for i in 0..self.num_hashes {
            let pos = self.hash_position(item, i);
            let word = pos / 64;
            let bit = pos % 64;
            self.bits[word] |= 1u64 << bit;
        }
        self.count += 1;
    }

    /// Проверяет, возможно ли наличие элемента.
    /// false = ТОЧНО нет. true = возможно да.
    pub fn maybe_contains(&self, item: u64) -> bool {
        for i in 0..self.num_hashes {
            let pos = self.hash_position(item, i);
            let word = pos / 64;
            let bit = pos % 64;
            if self.bits[word] & (1u64 << bit) == 0 {
                return false;
            }
        }
        true
    }

    /// Число вставленных элементов.
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Примерный false positive rate.
    pub fn estimated_fp_rate(&self) -> f64 {
        let m = self.num_bits as f64;
        let k = self.num_hashes as f64;
        let n = self.count as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    /// Double hashing: h(item, i) = (h1 + i * h2) mod m
    fn hash_position(&self, item: u64, i: u32) -> usize {
        let h1 = self.fnv1a(item);
        let h2 = self.murmur_mix(item);
        ((h1.wrapping_add((i as u64).wrapping_mul(h2))) % self.num_bits as u64) as usize
    }

    /// FNV-1a inspired hash.
    fn fnv1a(&self, mut val: u64) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for _ in 0..8 {
            hash ^= val & 0xFF;
            hash = hash.wrapping_mul(0x100000001b3);
            val >>= 8;
        }
        hash
    }

    /// Murmur-inspired mix for second hash.
    fn murmur_mix(&self, mut val: u64) -> u64 {
        val ^= val >> 33;
        val = val.wrapping_mul(0xff51afd7ed558ccd);
        val ^= val >> 33;
        val = val.wrapping_mul(0xc4ceb9fe1a85ec53);
        val ^= val >> 33;
        val | 1 // Ensure odd (coprime with power-of-2 modulus)
    }
}

/// Хэширует контекст (symbol + regime + session) в u64.
pub fn context_hash(symbol: &str, regime: &str, session: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in symbol.bytes().chain(b":".iter().copied())
        .chain(regime.bytes()).chain(b":".iter().copied())
        .chain(session.bytes())
    {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bf = BloomFilter::new(1024, 5);
        bf.insert(42);
        bf.insert(123);
        
        assert!(bf.maybe_contains(42));
        assert!(bf.maybe_contains(123));
    }

    #[test]
    fn test_not_contains() {
        let mut bf = BloomFilter::new(10_000, 7);
        bf.insert(42);
        // 99999 was never inserted — should probably be false
        // (with 10K bits and 1 element, FP rate is near zero)
        assert!(!bf.maybe_contains(99999), "Should not contain 99999");
    }

    #[test]
    fn test_optimal_creation() {
        let bf = BloomFilter::optimal(10_000, 0.001);
        assert!(bf.num_bits > 100_000, "Should have >100K bits for 10K items @ 0.1% FP");
        assert!(bf.num_hashes >= 7, "Should use >=7 hashes");
    }

    #[test]
    fn test_context_hash_deterministic() {
        let h1 = context_hash("BTCUSDT", "trending_up", "london");
        let h2 = context_hash("BTCUSDT", "trending_up", "london");
        assert_eq!(h1, h2, "Same input should give same hash");
    }

    #[test]
    fn test_context_hash_differs() {
        let h1 = context_hash("BTCUSDT", "trending_up", "london");
        let h2 = context_hash("ETHUSDT", "trending_up", "london");
        assert_ne!(h1, h2, "Different symbols should give different hashes");
    }

    #[test]
    fn test_fp_rate_low_with_few_items() {
        let mut bf = BloomFilter::optimal(10_000, 0.001);
        for i in 0..100 {
            bf.insert(i);
        }
        assert!(bf.estimated_fp_rate() < 0.0001, "FP rate should be very low with 100/10K items");
    }
}
