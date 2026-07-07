use std::time::Duration;
use crate::{config, engine};

pub async fn run_risk_loop() {
    println!("[Risk Agent] Initializing full Risk Gate (Kelly + ATR + BucketCap + KillSwitch + AutoRamp)...");

    let mut risk_gate = engine::RiskGate::new(config::MAX_EXPOSURE_USD);
    let mut tick_count: u64 = 0;

    loop {
        tick_count += 1;

        // Derive exposure from RiskGate state (bankroll - absolute daily drawdown)
        let current_exposure = risk_gate.kill_switch.bankroll + risk_gate.kill_switch.daily_pnl;
        if !engine::is_exposure_safe(current_exposure, config::MAX_EXPOSURE_USD) {
            println!("[Risk Agent] EXPOSURE CRITICAL. HALTING NEW TRADES.");
            tokio::time::sleep(Duration::from_millis(config::RISK_POLL_INTERVAL_MS)).await;
            continue;
        }

        // ── Full RiskGate evaluation on every tick ──
        // Simulate incoming trade request (in production, read from IPC)
        let symbol = "WBTC";
        let win_rate = 0.62;       // Would come from rolling stats
        let avg_loss_r = 1.0;     // Average loss in R-multiples
        let regime = engine::MarketRegime::Calm; // Would come from regime detector

        match risk_gate.evaluate(symbol, win_rate, avg_loss_r, regime) {
            Ok(position_size) => {
                println!("[Risk Agent] ✅ Trade APPROVED for {} | Size: ${:.2} | Regime: {:?}",
                         symbol, position_size, regime);
            }
            Err(reason) => {
                println!("[Risk Agent] 🚫 Trade BLOCKED: {}", reason);
            }
        }

        // ── Auto-Ramp evaluation every 100 ticks ──
        if tick_count.is_multiple_of(100) {
            let promoted = risk_gate.auto_ramp.evaluate_promotion(
                12,   // closed_trades_96h
                50.0, // pnl_7d (positive)
                0,    // kill_switch_incidents_96h
                0,    // bucket_breaches_96h
            );
            if promoted {
                println!("[Risk Agent] 🚀 AUTO-RAMP: Promoted to Phase {}!",
                         risk_gate.auto_ramp.current_stage);
            }
        }

        // ── Kill-Switch status ──
        if risk_gate.kill_switch.is_paused() {
            println!("[Risk Agent] ⛔ KILL-SWITCH ACTIVE — all trading paused");
        }

        tokio::time::sleep(Duration::from_millis(config::RISK_POLL_INTERVAL_MS)).await;
    }
}
