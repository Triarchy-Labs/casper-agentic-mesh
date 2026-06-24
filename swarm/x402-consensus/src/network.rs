use std::time::Duration;
use core_ipc::IpcBridge;
use crate::{config, engine};

pub async fn run_consensus_loop() {
    println!("[Consensus Node] Initializing Policy Governor (Signal + Trend + Regime voting)...");

    let mut ipc = IpcBridge::new();
    let governor = engine::PolicyGovernor::new();
    let mut last_timestamp = 0;

    loop {
        if let Some(state) = ipc.read_state()
            && state.timestamp > last_timestamp
        {
            last_timestamp = state.timestamp;

            // ── Construct agent votes from IPC state ──
            // In production, each sub-agent writes its own vote to IPC.
            // For now, we derive votes from the available IPC fields.
            let mut votes = Vec::new();

            // Signal agent: votes based on whether sniper detected a target
            if let Some(ref _target) = state.liquidation_target {
                votes.push(engine::AgentVote {
                    agent_name: "signal".to_string(),
                    action: engine::Action::Buy,
                    confidence: 0.7,
                    timestamp: state.timestamp,
                });
            } else {
                votes.push(engine::AgentVote {
                    agent_name: "signal".to_string(),
                    action: engine::Action::Wait,
                    confidence: 0.3,
                    timestamp: state.timestamp,
                });
            }

            // Trend agent: votes based on sentiment modifier
            let trend_action = if state.global_sentiment_modifier > 0.5 {
                engine::Action::Buy
            } else if state.global_sentiment_modifier < -0.5 {
                engine::Action::Sell
            } else {
                engine::Action::Wait
            };
            votes.push(engine::AgentVote {
                agent_name: "trend".to_string(),
                action: trend_action,
                confidence: (state.global_sentiment_modifier.abs() * 0.8).clamp(0.2, 0.8),
                timestamp: state.timestamp,
            });

            // Regime agent: votes based on risk_vote from IPC
            let regime_action = if state.risk_vote.unwrap_or(false) {
                engine::Action::Buy  // Risk approved
            } else {
                engine::Action::Wait // Risk not yet approved
            };
            votes.push(engine::AgentVote {
                agent_name: "regime".to_string(),
                action: regime_action,
                confidence: 0.55,
                timestamp: state.timestamp,
            });

            // ── Resolve via Policy Governor ──
            let decision = governor.resolve(&votes);

            match decision.action {
                engine::Action::Buy | engine::Action::Sell => {
                    println!("[Consensus Node] ✅ Governor Decision: {} (confidence: {:.2}, votes: {})",
                             decision.action, decision.confidence, decision.votes_received);
                    if decision.vetoed {
                        println!("[Consensus Node] ⚠️ VETOED: {:?}", decision.veto_reason);
                        let current_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        ipc.update_state(|s| {
                            s.consensus_reached = Some(false);
                            s.timestamp = current_ts;
                        });
                    } else {
                        println!("[Consensus Node] 🚀 Consensus REACHED — forwarding to execution!");
                        let current_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        ipc.update_state(|s| {
                            s.consensus_reached = Some(true);
                            s.timestamp = current_ts;
                        });
                    }
                }
                engine::Action::Wait => {
                    let reason = decision.veto_reason.as_deref().unwrap_or("No actionable signal");
                    println!("[Consensus Node] ⏳ WAIT — {}", reason);
                    let current_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                    ipc.update_state(|s| {
                        s.consensus_reached = Some(false);
                        s.timestamp = current_ts;
                    });
                }
            }

            // Legacy compat: still check old threshold
            let mut active_votes = 0;
            if state.sniper_vote.unwrap_or(false) { active_votes += 1; }
            if state.risk_vote.unwrap_or(false) { active_votes += 1; }
            if engine::check_consensus_reached(active_votes, config::VOTE_THRESHOLD) {
                println!("[Consensus Node] Legacy consensus also reached ({}/{})",
                         active_votes, config::VOTE_THRESHOLD);
            }
        }

        tokio::time::sleep(Duration::from_millis(config::CONSENSUS_TIMEOUT_MS)).await;
    }
}
