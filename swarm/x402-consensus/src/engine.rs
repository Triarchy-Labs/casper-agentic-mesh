use std::time::{SystemTime, UNIX_EPOCH};

// ── Agent Vote ─────────────────────────────────────────────────────────
/// A single sub-agent's vote in the consensus round.
/// Absorbed from Swarmbots ARCHITECTURE: Policy Governor pattern.
#[derive(Debug, Clone)]
pub struct AgentVote {
    pub agent_name: String,
    pub action: Action,
    pub confidence: f64,  // 0.0–1.0
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Buy,
    Sell,
    Wait,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Buy => write!(f, "BUY"),
            Action::Sell => write!(f, "SELL"),
            Action::Wait => write!(f, "WAIT"),
        }
    }
}

// ── Confidence Band ────────────────────────────────────────────────────
/// Per-symbol confidence filtering.
/// Absorbed from Swarmbots RISK-MODEL: Gate 6.
/// Rejects both under-confident AND over-confident signals.
#[derive(Debug, Clone)]
pub struct ConfidenceBand {
    pub low: f64,   // typical: 0.20
    pub high: f64,  // typical: 0.85
}

impl Default for ConfidenceBand {
    fn default() -> Self {
        Self { low: 0.20, high: 0.85 }
    }
}

impl ConfidenceBand {
    pub fn is_valid(&self, confidence: f64) -> bool {
        confidence >= self.low && confidence <= self.high
    }
}

// ── Governor Decision ──────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct GovernorDecision {
    pub action: Action,
    pub confidence: f64,
    pub vetoed: bool,
    pub veto_reason: Option<String>,
    pub votes_received: usize,
    pub timestamp: u64,
}

// ── Policy Governor ────────────────────────────────────────────────────
/// Multi-agent consensus governor.
/// Collects votes from Signal, Trend, and Regime agents.
/// Applies: confidence band filtering, trend veto, regime modulation.
/// Absorbed from Swarmbots ARCHITECTURE: Policy Governor.
#[derive(Debug, Clone)]
pub struct PolicyGovernor {
    pub confidence_band: ConfidenceBand,
    pub require_trend_alignment: bool,
    pub min_votes_for_action: usize,
}

impl Default for PolicyGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyGovernor {
    pub fn new() -> Self {
        Self {
            confidence_band: ConfidenceBand::default(),
            require_trend_alignment: true,
            min_votes_for_action: 2,
        }
    }

    /// Resolve votes from all sub-agents into a single decision.
    /// 
    /// Rules (from Swarmbots):
    /// 1. Filter out votes with confidence outside the band
    /// 2. Count remaining directional votes (Buy/Sell)
    /// 3. If trend_agent voted WAIT and a Signal voted counter-trend → VETO
    /// 4. Majority action wins; ties → WAIT
    pub fn resolve(&self, votes: &[AgentVote]) -> GovernorDecision {
        let now = now_epoch();

        // Step 1: Filter by confidence band
        let valid_votes: Vec<&AgentVote> = votes.iter()
            .filter(|v| self.confidence_band.is_valid(v.confidence))
            .collect();

        if valid_votes.is_empty() {
            return GovernorDecision {
                action: Action::Wait,
                confidence: 0.0,
                vetoed: false,
                veto_reason: Some("No votes passed confidence band".to_string()),
                votes_received: votes.len(),
                timestamp: now,
            };
        }

        // Step 2: Find the trend agent's vote (for veto logic)
        let trend_vote = valid_votes.iter()
            .find(|v| v.agent_name == "trend")
            .map(|v| v.action);

        // Step 3: Count directional votes
        let buy_votes: Vec<&&AgentVote> = valid_votes.iter().filter(|v| v.action == Action::Buy).collect();
        let sell_votes: Vec<&&AgentVote> = valid_votes.iter().filter(|v| v.action == Action::Sell).collect();

        let (majority_action, majority_count, avg_confidence) = if buy_votes.len() > sell_votes.len() {
            let avg_conf = buy_votes.iter().map(|v| v.confidence).sum::<f64>() / buy_votes.len() as f64;
            (Action::Buy, buy_votes.len(), avg_conf)
        } else if sell_votes.len() > buy_votes.len() {
            let avg_conf = sell_votes.iter().map(|v| v.confidence).sum::<f64>() / sell_votes.len() as f64;
            (Action::Sell, sell_votes.len(), avg_conf)
        } else {
            // Tie → WAIT
            return GovernorDecision {
                action: Action::Wait,
                confidence: 0.0,
                vetoed: false,
                veto_reason: Some("Vote tie — defaulting to WAIT".to_string()),
                votes_received: votes.len(),
                timestamp: now,
            };
        };

        // Step 4: Minimum vote threshold
        if majority_count < self.min_votes_for_action {
            return GovernorDecision {
                action: Action::Wait,
                confidence: avg_confidence,
                vetoed: false,
                veto_reason: Some(format!("Insufficient votes: {} < {}", majority_count, self.min_votes_for_action)),
                votes_received: votes.len(),
                timestamp: now,
            };
        }

        // Step 5: Trend veto — block counter-trend signals
        if self.require_trend_alignment
            && let Some(trend) = trend_vote {
                let counter_trend = match (majority_action, trend) {
                    (Action::Buy, Action::Sell) => true,
                    (Action::Sell, Action::Buy) => true,
                    _ => false,
                };
                if counter_trend {
                    return GovernorDecision {
                        action: Action::Wait,
                        confidence: avg_confidence,
                        vetoed: true,
                        veto_reason: Some(format!("Trend veto: {} signal against {} trend", majority_action, trend)),
                        votes_received: votes.len(),
                        timestamp: now,
                    };
                }
            }

        // All checks passed — execute
        GovernorDecision {
            action: majority_action,
            confidence: avg_confidence,
            vetoed: false,
            veto_reason: None,
            votes_received: votes.len(),
            timestamp: now,
        }
    }
}

// ── Legacy API (backward compat) ───────────────────────────────────────
pub fn check_consensus_reached(votes: usize, threshold: usize) -> bool {
    votes >= threshold
}

// ── Helpers ────────────────────────────────────────────────────────────
fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vote(name: &str, action: Action, confidence: f64) -> AgentVote {
        AgentVote {
            agent_name: name.to_string(),
            action,
            confidence,
            timestamp: 0,
        }
    }

    #[test]
    fn test_unanimous_buy() {
        let gov = PolicyGovernor::new();
        let votes = vec![
            make_vote("signal", Action::Buy, 0.7),
            make_vote("trend", Action::Buy, 0.6),
            make_vote("regime", Action::Buy, 0.5),
        ];
        let decision = gov.resolve(&votes);
        assert_eq!(decision.action, Action::Buy);
        assert!(!decision.vetoed);
    }

    #[test]
    fn test_trend_veto() {
        let gov = PolicyGovernor::new();
        let votes = vec![
            make_vote("signal", Action::Buy, 0.7),
            make_vote("trend", Action::Sell, 0.6),
            make_vote("regime", Action::Buy, 0.5),
        ];
        let decision = gov.resolve(&votes);
        assert_eq!(decision.action, Action::Wait);
        assert!(decision.vetoed);
    }

    #[test]
    fn test_confidence_too_low() {
        let gov = PolicyGovernor::new();
        let votes = vec![
            make_vote("signal", Action::Buy, 0.10),  // below band
            make_vote("trend", Action::Buy, 0.05),    // below band
            make_vote("regime", Action::Buy, 0.15),   // below band
        ];
        let decision = gov.resolve(&votes);
        assert_eq!(decision.action, Action::Wait);
    }

    #[test]
    fn test_confidence_too_high() {
        let gov = PolicyGovernor::new();
        let votes = vec![
            make_vote("signal", Action::Buy, 0.95),  // above band (overfit)
            make_vote("trend", Action::Buy, 0.90),    // above band
            make_vote("regime", Action::Buy, 0.50),   // valid
        ];
        // Only regime passes band → 1 vote < min 2 → WAIT
        let decision = gov.resolve(&votes);
        assert_eq!(decision.action, Action::Wait);
    }

    #[test]
    fn test_vote_tie() {
        let gov = PolicyGovernor::new();
        let votes = vec![
            make_vote("signal", Action::Buy, 0.7),
            make_vote("trend", Action::Sell, 0.6),
            make_vote("regime", Action::Wait, 0.5),
        ];
        let decision = gov.resolve(&votes);
        assert_eq!(decision.action, Action::Wait);
    }

    #[test]
    fn test_legacy_compat() {
        assert!(check_consensus_reached(3, 2));
        assert!(!check_consensus_reached(1, 2));
    }
}
