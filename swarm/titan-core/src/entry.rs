// src/modules/entry.rs
// ═══════════════════════════════════════════════════════════════
// ENTRY GATE PIPELINE — Pure Decision Logic for Trade Entry
// ═══════════════════════════════════════════════════════════════
// Вынесено из main.rs (gates 1-7) в чистый модуль.
//
// ПРИНЦИП: Этот модуль содержит ТОЛЬКО чистую логику (no async, no API).
// Все side-effects (API calls, RwLock writes) остаются в main.rs.
// Модуль принимает данные → возвращает EntryVerdict.

/// Конфигурация Entry Pipeline
pub struct EntryConfig {
    /// Максимум позиций на один head
    pub max_positions_per_head: usize,
    /// Максимум глобальных позиций
    pub max_global_positions: usize,
    /// Максимум loss streak per symbol
    pub max_loss_streak: u32,
    /// Total exposure cap (% от баланса)
    pub total_exposure_cap_pct: f64,
    /// BTC dead zone threshold (FORENSIC-02)
    pub btc_dead_zone: f64,
    /// Imbalance min/max reject thresholds
    pub imbalance_reject_min: f64,
    pub imbalance_reject_max: f64,
    /// LONG requires imbalance > this
    pub imbalance_long_min: f64,
    /// SHORT requires imbalance < this
    pub imbalance_short_max: f64,
}

impl Default for EntryConfig {
    fn default() -> Self {
        Self {
            max_positions_per_head: 3,
            max_global_positions: 9,
            max_loss_streak: 2,
            total_exposure_cap_pct: 0.80,
            btc_dead_zone: 0.5,
            imbalance_reject_min: 0.1,
            imbalance_reject_max: 10.0,
            imbalance_long_min: 0.5,
            imbalance_short_max: 2.0,
        }
    }
}

/// Результат проверки всех гейтов
#[derive(Debug, Clone)]
#[allow(dead_code)] // reason fields used via Debug + future dashboard reporting
pub enum EntryVerdict {
    /// Все гейты пройдены — можно входить
    Approved {
        side: String,       // "Buy" or "Sell"
        reason: String,
    },
    /// Заблокировано — причина отказа
    Rejected {
        gate: String,       // Какой гейт заблокировал
        reason: String,
    },
}

/// Данные для принятия решения (собираются в main.rs, передаются сюда)
pub struct EntryContext {
    pub daily_loss: f64,
    pub session_limit: f64,
    pub symbol_loss_streak: u32,
    pub head_position_count: usize,
    pub global_position_count: usize,
    pub symbol_already_owned: bool,
    pub verdict: String,          // "LONG" | "SHORT" | "NONE" | "API_BLIND" | "SWARM_DEAD"
    pub symbol: String,            // V11: needed for correlation bucket check
    pub score: f64,
    pub btc_score: f64,
    pub imbalance_ratio: f64,
    pub existing_total_margin: f64,
    pub available_balance: f64,
    pub new_margin_size: f64,
    pub is_held_by_other_bot: bool,
    /// V11: Symbols of existing positions (for correlation bucket check)
    pub existing_position_symbols: Vec<String>,
}

pub struct EntryPipeline;

impl EntryPipeline {
    /// Прогоняет все гейты. Pure function — no side effects.
    pub fn evaluate(ctx: &EntryContext, config: &EntryConfig) -> EntryVerdict {
        // Gate 0: Invalid verdict
        if ctx.verdict == "SWARM_DEAD" || ctx.verdict == "API_BLIND" {
            return EntryVerdict::Rejected {
                gate: "G0_VERDICT".to_string(),
                reason: format!("Invalid verdict: {}", ctx.verdict),
            };
        }

        // Gate 1: Session loss limit
        if ctx.daily_loss >= ctx.session_limit {
            return EntryVerdict::Rejected {
                gate: "G1_SESSION_LOSS".to_string(),
                reason: format!("Daily loss ${:.2} >= limit ${:.2}", ctx.daily_loss, ctx.session_limit),
            };
        }

        // Gate 2: Per-symbol loss streak
        if ctx.symbol_loss_streak >= config.max_loss_streak {
            return EntryVerdict::Rejected {
                gate: "G2_LOSS_STREAK".to_string(),
                reason: format!("Symbol streak {} >= {}", ctx.symbol_loss_streak, config.max_loss_streak),
            };
        }

        // Gate 3: Position limits + ownership
        if ctx.head_position_count >= config.max_positions_per_head {
            return EntryVerdict::Rejected {
                gate: "G3_HEAD_LIMIT".to_string(),
                reason: format!("Head has {} positions (max {})", ctx.head_position_count, config.max_positions_per_head),
            };
        }
        if ctx.global_position_count >= config.max_global_positions {
            return EntryVerdict::Rejected {
                gate: "G3_GLOBAL_LIMIT".to_string(),
                reason: format!("Global {} positions (max {})", ctx.global_position_count, config.max_global_positions),
            };
        }
        if ctx.symbol_already_owned {
            return EntryVerdict::Rejected {
                gate: "G3_DUPLICATE".to_string(),
                reason: "Symbol already owned or reserved".to_string(),
            };
        }

        // Gate 4: Anti-Duplication (другой бот Роя)
        if ctx.is_held_by_other_bot {
            return EntryVerdict::Rejected {
                gate: "G4_SWARM_DUP".to_string(),
                reason: "Held by another swarm bot".to_string(),
            };
        }

        // Gate 4.5: V11 Correlation Buckets (Swarmbots-inspired)
        // Prevent opening correlated positions (BTC+ETH, SOL+DOGE+PEPE)
        if let Some(bucket_name) = Self::get_bucket(&ctx.symbol) {
            for existing_sym in &ctx.existing_position_symbols {
                if existing_sym != &ctx.symbol {
                    if let Some(existing_bucket) = Self::get_bucket(existing_sym) {
                        if bucket_name == existing_bucket {
                            return EntryVerdict::Rejected {
                                gate: "G4_5_CORR_BUCKET".to_string(),
                                reason: format!("Bucket '{bucket_name}' already has {existing_sym} — correlated"),
                            };
                        }
                    }
                }
            }
        }

        // Gate 5: Imbalance check (FORENSIC-04: direction-aware)
        let imbalance_ok = if ctx.imbalance_ratio < config.imbalance_reject_min || ctx.imbalance_ratio > config.imbalance_reject_max {
            false
        } else if ctx.verdict == "LONG" {
            ctx.imbalance_ratio > config.imbalance_long_min
        } else if ctx.verdict == "SHORT" {
            ctx.imbalance_ratio < config.imbalance_short_max
        } else {
            true
        };
        if !imbalance_ok {
            return EntryVerdict::Rejected {
                gate: "G5_IMBALANCE".to_string(),
                reason: format!("Imbalance {:.2} wrong for {}", ctx.imbalance_ratio, ctx.verdict),
            };
        }

        // Gate 6: BTC alignment (FORENSIC-02: dead zone)
        let btc_bullish = ctx.btc_score > config.btc_dead_zone;
        let btc_bearish = ctx.btc_score < -config.btc_dead_zone;
        let btc_neutral = !btc_bullish && !btc_bearish;

        let aligned = (ctx.verdict == "LONG" && (btc_bullish || btc_neutral))
                    || (ctx.verdict == "SHORT" && (btc_bearish || btc_neutral));
        if !aligned {
            return EntryVerdict::Rejected {
                gate: "G6_BTC_ALIGN".to_string(),
                reason: format!("{} vs BTC score {:.1} — misaligned", ctx.verdict, ctx.btc_score),
            };
        }

        // Gate 7: Total exposure cap (FORENSIC-07)
        if ctx.existing_total_margin + ctx.new_margin_size > ctx.available_balance * config.total_exposure_cap_pct {
            return EntryVerdict::Rejected {
                gate: "G7_EXPOSURE".to_string(),
                reason: format!("Margin ${:.2}+${:.2} > {:.0}% of ${:.2}",
                    ctx.existing_total_margin, ctx.new_margin_size,
                    config.total_exposure_cap_pct * 100.0, ctx.available_balance),
            };
        }

        // ✅ All gates passed
        let side = if ctx.verdict == "LONG" { "Buy" } else { "Sell" };
        EntryVerdict::Approved {
            side: side.to_string(),
            reason: format!("Score:{:.1} BTC:{:.1} Imb:{:.2}", ctx.score, ctx.btc_score, ctx.imbalance_ratio),
        }
    }

    /// V11.0.1: Expanded correlation bucket lookup (8 → 20 coins)
    /// Coins in the same bucket tend to move together — only one position per bucket allowed
    fn get_bucket(symbol: &str) -> Option<&'static str> {
        match symbol {
            // Major Layer-1s — highest correlation pair in crypto
            "BTCUSDT" | "ETHUSDT" => Some("major"),
            // Meme ecosystem — move in lockstep during hype waves
            "DOGEUSDT" | "PEPEUSDT" | "SHIBUSDT" | "FLOKIUSDT" | "BONKUSDT" => Some("meme"),
            // Solana ecosystem — correlated via SOL price dependency
            "SOLUSDT" | "WIFUSDT" | "JUPUSDT" | "RAYUSDT" => Some("sol_eco"),
            // Infrastructure alts — legacy L1s, move together on "alt season" rotation
            "ADAUSDT" | "XRPUSDT" | "DOTUSDT" | "AVAXUSDT" | "MATICUSDT" => Some("alt_infra"),
            // DeFi blue chips — correlated through TVL flows
            "AAVEUSDT" | "UNIUSDT" | "LINKUSDT" => Some("defi"),
            // AI & Data — narrative-driven correlation
            "FETUSDT" | "RENDERUSDT" | "NEARUSDT" => Some("ai_data"),
            _ => None, // Independent — no bucket restriction
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// UNIT TESTS — Pure function tests, no async/mocking needed
// ═══════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: создаёт "идеальный" LONG контекст (все гейты пройдены)
    fn make_valid_long_ctx() -> EntryContext {
        EntryContext {
            daily_loss: 0.0,
            session_limit: 18.0,
            symbol_loss_streak: 0,
            head_position_count: 0,
            global_position_count: 0,
            symbol_already_owned: false,
            verdict: "LONG".to_string(),
            symbol: "TESTUSDT".to_string(),
            score: 7.5,
            btc_score: 1.5,
            imbalance_ratio: 1.2,
            existing_total_margin: 10.0,
            available_balance: 100.0,
            new_margin_size: 5.0,
            is_held_by_other_bot: false,
            existing_position_symbols: vec![],
        }
    }

    #[test]
    fn test_g0_swarm_dead_rejected() {
        let mut ctx = make_valid_long_ctx();
        ctx.verdict = "SWARM_DEAD".to_string();
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G0_VERDICT"),
            _ => panic!("Expected G0 rejection"),
        }
    }

    #[test]
    fn test_g0_api_blind_rejected() {
        let mut ctx = make_valid_long_ctx();
        ctx.verdict = "API_BLIND".to_string();
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G0_VERDICT"),
            _ => panic!("Expected G0 rejection"),
        }
    }

    #[test]
    fn test_g1_session_loss_exceeded() {
        let mut ctx = make_valid_long_ctx();
        ctx.daily_loss = 20.0; // > session_limit of 18.0
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G1_SESSION_LOSS"),
            _ => panic!("Expected G1 rejection"),
        }
    }

    #[test]
    fn test_g2_loss_streak_exceeded() {
        let mut ctx = make_valid_long_ctx();
        ctx.symbol_loss_streak = 3; // >= max_loss_streak (2)
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G2_LOSS_STREAK"),
            _ => panic!("Expected G2 rejection"),
        }
    }

    #[test]
    fn test_g3_head_limit() {
        let mut ctx = make_valid_long_ctx();
        ctx.head_position_count = 3; // >= max_positions_per_head (3)
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G3_HEAD_LIMIT"),
            _ => panic!("Expected G3 head rejection"),
        }
    }

    #[test]
    fn test_g3_global_limit() {
        let mut ctx = make_valid_long_ctx();
        ctx.global_position_count = 9; // >= max_global (9)
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G3_GLOBAL_LIMIT"),
            _ => panic!("Expected G3 global rejection"),
        }
    }

    #[test]
    fn test_g3_duplicate() {
        let mut ctx = make_valid_long_ctx();
        ctx.symbol_already_owned = true;
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G3_DUPLICATE"),
            _ => panic!("Expected G3 dup rejection"),
        }
    }

    #[test]
    fn test_g4_swarm_dup() {
        let mut ctx = make_valid_long_ctx();
        ctx.is_held_by_other_bot = true;
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G4_SWARM_DUP"),
            _ => panic!("Expected G4 rejection"),
        }
    }

    #[test]
    fn test_g5_imbalance_long_too_low() {
        let mut ctx = make_valid_long_ctx();
        ctx.imbalance_ratio = 0.3; // < imbalance_long_min (0.5)
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G5_IMBALANCE"),
            _ => panic!("Expected G5 rejection for LONG"),
        }
    }

    #[test]
    fn test_g5_imbalance_short_too_high() {
        let mut ctx = make_valid_long_ctx();
        ctx.verdict = "SHORT".to_string();
        ctx.btc_score = -1.5; // align BTC for SHORT
        ctx.imbalance_ratio = 3.0; // > imbalance_short_max (2.0)
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G5_IMBALANCE"),
            _ => panic!("Expected G5 rejection for SHORT"),
        }
    }

    #[test]
    fn test_g6_btc_misaligned_long() {
        let mut ctx = make_valid_long_ctx();
        ctx.btc_score = -2.0; // bearish BTC vs LONG
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G6_BTC_ALIGN"),
            _ => panic!("Expected G6 rejection"),
        }
    }

    #[test]
    fn test_g7_exposure_cap() {
        let mut ctx = make_valid_long_ctx();
        ctx.existing_total_margin = 75.0;
        ctx.new_margin_size = 10.0; // 75+10=85 > 80% of 100
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Rejected { gate, .. } => assert_eq!(gate, "G7_EXPOSURE"),
            _ => panic!("Expected G7 rejection"),
        }
    }

    #[test]
    fn test_approved_long() {
        let ctx = make_valid_long_ctx();
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Approved { side, .. } => assert_eq!(side, "Buy"),
            _ => panic!("Expected LONG approval"),
        }
    }

    #[test]
    fn test_approved_short() {
        let mut ctx = make_valid_long_ctx();
        ctx.verdict = "SHORT".to_string();
        ctx.btc_score = -1.5;
        ctx.imbalance_ratio = 1.5;
        let result = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        match result {
            EntryVerdict::Approved { side, .. } => assert_eq!(side, "Sell"),
            _ => panic!("Expected SHORT approval"),
        }
    }

    #[test]
    fn test_btc_neutral_allows_both() {
        let mut ctx = make_valid_long_ctx();
        ctx.btc_score = 0.0; // neutral — should allow LONG
        let r1 = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        assert!(matches!(r1, EntryVerdict::Approved { .. }));

        ctx.verdict = "SHORT".to_string();
        ctx.imbalance_ratio = 1.5;
        let r2 = EntryPipeline::evaluate(&ctx, &EntryConfig::default());
        assert!(matches!(r2, EntryVerdict::Approved { .. }));
    }

    #[test]
    fn test_custom_config() {
        let mut config = EntryConfig::default();
        config.max_positions_per_head = 5; // more permissive
        let mut ctx = make_valid_long_ctx();
        ctx.head_position_count = 4; // would fail default (3) but pass custom (5)
        let result = EntryPipeline::evaluate(&ctx, &config);
        assert!(matches!(result, EntryVerdict::Approved { .. }));
    }

    // ═══ V11.0.1 CORRELATION BUCKET TESTS ═══

    #[test]
    fn test_bucket_major() {
        assert_eq!(EntryPipeline::get_bucket("BTCUSDT"), Some("major"));
        assert_eq!(EntryPipeline::get_bucket("ETHUSDT"), Some("major"));
    }

    #[test]
    fn test_bucket_meme() {
        assert_eq!(EntryPipeline::get_bucket("DOGEUSDT"), Some("meme"));
        assert_eq!(EntryPipeline::get_bucket("PEPEUSDT"), Some("meme"));
        assert_eq!(EntryPipeline::get_bucket("SHIBUSDT"), Some("meme"));
        assert_eq!(EntryPipeline::get_bucket("BONKUSDT"), Some("meme"));
    }

    #[test]
    fn test_bucket_sol_eco() {
        assert_eq!(EntryPipeline::get_bucket("SOLUSDT"), Some("sol_eco"));
        assert_eq!(EntryPipeline::get_bucket("WIFUSDT"), Some("sol_eco"));
        assert_eq!(EntryPipeline::get_bucket("JUPUSDT"), Some("sol_eco"));
    }

    #[test]
    fn test_bucket_defi_and_ai() {
        assert_eq!(EntryPipeline::get_bucket("AAVEUSDT"), Some("defi"));
        assert_eq!(EntryPipeline::get_bucket("LINKUSDT"), Some("defi"));
        assert_eq!(EntryPipeline::get_bucket("FETUSDT"), Some("ai_data"));
        assert_eq!(EntryPipeline::get_bucket("NEARUSDT"), Some("ai_data"));
    }

    #[test]
    fn test_bucket_unknown_is_independent() {
        assert_eq!(EntryPipeline::get_bucket("RANDOMCOINUSDT"), None);
        assert_eq!(EntryPipeline::get_bucket("CHILLGUYUSDT"), None);
    }
}
