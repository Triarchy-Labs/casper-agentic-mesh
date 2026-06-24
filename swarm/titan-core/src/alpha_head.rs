// src/modules/alpha_head.rs
use std::collections::HashMap;

/// Альфа-Голова (Бывший Titan Monster). 
/// Работает ИСКЛЮЧИТЕЛЬНО на 5-минутном таймфрейме.
/// Впитал в себя Пиранью (Dual Trailing Protocol).
pub struct AlphaGhostHead {
    pub active_positions: HashMap<String, GhostPosition>,
}

#[allow(dead_code)] // symbol/qty used in Debug logging + future serialization
pub struct GhostPosition {
    pub symbol: String,
    pub entry_price: f64,
    pub side: String,
    pub qty: f64,
    pub ghost_sl: f64,        // [SOFT] Невидимый локальный стоп для снайперского выхода
    pub hardware_sl: f64,     // [HARD] Wide exchange SL fallback (Щит от отключения света/проливов ММ)
    pub highest_profit_price: f64,
}

impl Default for AlphaGhostHead {
    fn default() -> Self {
        Self::new()
    }
}

impl AlphaGhostHead {
    pub fn new() -> Self {
        AlphaGhostHead {
            active_positions: HashMap::new(),
        }
    }

    /// Регистрирует новую позицию. Hardware SL ставится широко за стенку. Ghost SL ставится на 1 ATR от точки входа.
    pub fn register_entry(&mut self, symbol: &str, price: f64, side: &str, qty: f64, hard_sl: f64, atr: f64) {
        let ghost_sl = if side == "Buy" {
            hard_sl.max(price - (1.0 * atr)) // Теневой стоп на 1 ATR, но не ниже/выше аппаратного
        } else {
            hard_sl.min(price + (1.0 * atr))
        };

        self.active_positions.insert(
            symbol.to_string(),
            GhostPosition {
                symbol: symbol.to_string(),
                entry_price: price,
                side: side.to_string(),
                qty,
                ghost_sl,
                hardware_sl: hard_sl,
                highest_profit_price: price,
            }
        );
        tracing::info!(symbol = %symbol, hard_sl = format!("{hard_sl:.4}").as_str(), ghost_sl = format!("{ghost_sl:.4}").as_str(), "🦈 [PIRANHA DNA] Запущен");
    }

    /// Piranha Dual Trailing Protocol: Оценивает прибыль и двигает сразу два стопа (Matryoshka Protocol)
    /// 
    /// ⚠️ V11.0.1 DEPRECATED: This function is NOT called in the main loop.
    /// Exit logic is now handled by TrailingEngine::calculate_trailing_sl() + check_adverse_selection().
    /// register_entry() and hardware_sl sync (BUG-01) are still active.
    /// Kept for potential re-integration in V12 (Ghost SL + Trailing hybrid).
    #[allow(dead_code)]
    pub fn monitor_ghost_status(&mut self, symbol: &str, current_price: f64, atr: f64) -> Option<&str> {
        if let Some(pos) = self.active_positions.get_mut(symbol) {
            
            let (is_profit_move, profit_pct) = if pos.side == "Buy" {
                (current_price > pos.highest_profit_price, (current_price - pos.entry_price) / pos.entry_price * 100.0)
            } else {
                (current_price < pos.highest_profit_price, (pos.entry_price - current_price) / pos.entry_price * 100.0)
            };

            if is_profit_move {
                pos.highest_profit_price = current_price;
                
                // === GENESIS EXIT PROTOCOL (MATRYOSHKA STOPS) ===
                // Фаза 1 (Новорожденный): до прохождения 1.5 ATR профита мы вообще не трогаем стопы. Даем рынку дышать.
                let atr_target_pct = (1.5 * atr / pos.entry_price) * 100.0;
                
                // Фаза 2 (Щит) и Фаза 3 (Пиранья ATR-Трал) активируются после прохождения 1.5 ATR
                if profit_pct > atr_target_pct {
                    
                    // Фаза 2: Жесткий Profit Lock (Телепортация теневого стопа в +0.5 ATR прибыли)
                    let profit_lock_price = if pos.side == "Buy" {
                        pos.entry_price + (0.5 * atr)
                    } else {
                        pos.entry_price - (0.5 * atr)
                    };
                    
                    // Фаза 3: Адаптивная Пиранья (Зазор = 1.5 * ATR). Не дает выбить случайным чихом.
                    let atr_trail_gap = 1.5 * atr;
                    let atr_trail_sl = if pos.side == "Buy" {
                        current_price - atr_trail_gap
                    } else {
                        current_price + atr_trail_gap
                    };

                    // Умный Теневой Стоп выбирает лучшее из двух: или Жесткий +1% Блок, или подтянутый ATR Трал.
                    let new_ghost_sl = if pos.side == "Buy" {
                        profit_lock_price.max(atr_trail_sl)
                    } else {
                        profit_lock_price.min(atr_trail_sl)
                    };
                    
                    // Аппаратный (Hardware) подтягиваем с люфтом в 1.5% от теневого (защита на бирже)
                    let new_hard_sl = if pos.side == "Buy" { new_ghost_sl * 0.985 } else { new_ghost_sl * 1.015 };

                    // Храповик (никогда не двигаем стоп обратно в минус)
                    if pos.side == "Buy" {
                        if new_ghost_sl > pos.ghost_sl { 
                            pos.ghost_sl = new_ghost_sl; 
                            tracing::info!(symbol = %symbol, ghost_sl = format!("{:.4}", pos.ghost_sl).as_str(), "👻 [GENESIS LOCK] Профит 100% залочен");
                        }
                        if new_hard_sl > pos.hardware_sl {
                            pos.hardware_sl = new_hard_sl;
                            tracing::info!(hardware_sl = format!("{:.4}", pos.hardware_sl).as_str(), "🛡️ [IRON WALL] SL adjusted");
                        }
                    } else {
                        if new_ghost_sl < pos.ghost_sl { 
                            pos.ghost_sl = new_ghost_sl;
                            tracing::info!(symbol = %symbol, ghost_sl = format!("{:.4}", pos.ghost_sl).as_str(), "👻 [GENESIS LOCK] Шорт-профит залочен");
                        }
                        if new_hard_sl < pos.hardware_sl {
                            pos.hardware_sl = new_hard_sl;
                            tracing::info!(hardware_sl = format!("{:.4}", pos.hardware_sl).as_str(), "🛡️ [IRON WALL] SL adjusted");
                        }
                    }
                }
            }

            // ПРОВЕРКА УДАРА ПО СТОПАМ
            let is_ghost_triggered = if pos.side == "Buy" {
                current_price <= pos.ghost_sl
            } else {
                current_price >= pos.ghost_sl
            };

            let is_hard_triggered = if pos.side == "Buy" {
                current_price <= pos.hardware_sl
            } else {
                current_price >= pos.hardware_sl
            };

            if is_ghost_triggered {
                tracing::error!("🚨 [PIRANHA STRIKE] Цена пробила теневой барьер! Экстренный Market Close!");
                return Some("TRIGGER_MARKET_CLOSE"); 
            }
            
            if is_hard_triggered {
                tracing::error!("💀 [FATAL] Hardware SL breached проскальзыванием раньше Ghost SL. Бой окончен.");
                return Some("HARDWARE_FATALITY");
            }
        }
        None
    }
}
