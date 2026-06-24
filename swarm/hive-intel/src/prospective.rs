/// Prospective Memory: "Remembering to do something in the future."
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/owm/prospective.py (87 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// В трейдинге это — заранее спланированные действия:
///   "Если цена пробьёт X → открыть лонг"
///   "Если funding > 0.05% → сократить позицию"
///
/// Механизм:
///   1. evaluate_trigger — проверяет набор условий (AND логика) против текущего контекста
///   2. record_outcome — записывает результат исполнения плана (PnL, timestamp)
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════

/// Comparison operator for trigger conditions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompareOp {
    Gt,   // >
    Lt,   // <
    Gte,  // >=
    Lte,  // <=
    Eq,   // ==
}

impl CompareOp {
    /// Parse from string (порт: prospective.py:50-63)
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "gt" => Some(Self::Gt),
            "lt" => Some(Self::Lt),
            "gte" => Some(Self::Gte),
            "lte" => Some(Self::Lte),
            "eq" => Some(Self::Eq),
            _ => None,
        }
    }

    fn apply(&self, actual: f64, value: f64) -> bool {
        match self {
            Self::Gt => actual > value,
            Self::Lt => actual < value,
            Self::Gte => actual >= value,
            Self::Lte => actual <= value,
            Self::Eq => (actual - value).abs() < f64::EPSILON,
        }
    }
}

/// A single trigger condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub field: String,
    pub op: CompareOp,
    pub value: f64,
}

/// A prospective plan with conditions and optional outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectivePlan {
    pub id: String,
    pub conditions: Vec<TriggerCondition>,
    pub action: String,        // e.g. "LONG", "SHORT", "REDUCE"
    pub status: PlanStatus,
    pub outcome: Option<PlanOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlanStatus {
    Pending,
    Triggered,
    Completed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOutcome {
    pub pnl: f64,
    pub profitable: bool,
    pub recorded_at_epoch: u64,
}

// ═══════════════════════════════════════════════════════════════
// Core Functions (порт: prospective.py:11-86)
// ═══════════════════════════════════════════════════════════════

/// Check whether a plan's trigger conditions are met by current context.
/// All conditions must be satisfied (AND logic).
///
/// Порт: prospective.py:11-47 (evaluate_trigger)
///
/// * `context` — HashMap of current market/system state values
pub fn evaluate_trigger(
    plan: &ProspectivePlan,
    context: &std::collections::HashMap<String, f64>,
) -> bool {
    if plan.conditions.is_empty() {
        return false;
    }

    for cond in &plan.conditions {
        match context.get(&cond.field) {
            Some(&actual) => {
                if !cond.op.apply(actual, cond.value) {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}

/// Record outcome after plan execution. Returns updated plan.
///
/// Порт: prospective.py:66-86 (record_outcome)
pub fn record_outcome(plan: &ProspectivePlan, actual_pnl: f64, epoch: u64) -> ProspectivePlan {
    let mut result = plan.clone();
    result.outcome = Some(PlanOutcome {
        pnl: actual_pnl,
        profitable: actual_pnl > 0.0,
        recorded_at_epoch: epoch,
    });
    result.status = PlanStatus::Completed;
    result
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_plan(conditions: Vec<TriggerCondition>) -> ProspectivePlan {
        ProspectivePlan {
            id: "test_plan".to_string(),
            conditions,
            action: "LONG".to_string(),
            status: PlanStatus::Pending,
            outcome: None,
        }
    }

    #[test]
    fn test_trigger_all_met() {
        let plan = make_plan(vec![
            TriggerCondition { field: "price".into(), op: CompareOp::Gt, value: 100.0 },
            TriggerCondition { field: "funding".into(), op: CompareOp::Lt, value: 0.0 },
        ]);
        let mut ctx = HashMap::new();
        ctx.insert("price".into(), 105.0);
        ctx.insert("funding".into(), -0.001);

        assert!(evaluate_trigger(&plan, &ctx));
    }

    #[test]
    fn test_trigger_one_fails() {
        let plan = make_plan(vec![
            TriggerCondition { field: "price".into(), op: CompareOp::Gt, value: 100.0 },
            TriggerCondition { field: "funding".into(), op: CompareOp::Lt, value: 0.0 },
        ]);
        let mut ctx = HashMap::new();
        ctx.insert("price".into(), 105.0);
        ctx.insert("funding".into(), 0.001); // positive → fails Lt condition

        assert!(!evaluate_trigger(&plan, &ctx));
    }

    #[test]
    fn test_trigger_missing_field() {
        let plan = make_plan(vec![
            TriggerCondition { field: "price".into(), op: CompareOp::Gt, value: 100.0 },
        ]);
        let ctx = HashMap::new(); // empty context
        assert!(!evaluate_trigger(&plan, &ctx));
    }

    #[test]
    fn test_trigger_empty_conditions() {
        let plan = make_plan(vec![]);
        let ctx = HashMap::new();
        assert!(!evaluate_trigger(&plan, &ctx));
    }

    #[test]
    fn test_all_operators() {
        let mut ctx = HashMap::new();
        ctx.insert("x".into(), 5.0);

        let gt = make_plan(vec![TriggerCondition { field: "x".into(), op: CompareOp::Gt, value: 4.0 }]);
        assert!(evaluate_trigger(&gt, &ctx));

        let lt = make_plan(vec![TriggerCondition { field: "x".into(), op: CompareOp::Lt, value: 6.0 }]);
        assert!(evaluate_trigger(&lt, &ctx));

        let gte = make_plan(vec![TriggerCondition { field: "x".into(), op: CompareOp::Gte, value: 5.0 }]);
        assert!(evaluate_trigger(&gte, &ctx));

        let lte = make_plan(vec![TriggerCondition { field: "x".into(), op: CompareOp::Lte, value: 5.0 }]);
        assert!(evaluate_trigger(&lte, &ctx));

        let eq = make_plan(vec![TriggerCondition { field: "x".into(), op: CompareOp::Eq, value: 5.0 }]);
        assert!(evaluate_trigger(&eq, &ctx));
    }

    #[test]
    fn test_record_outcome_profit() {
        let plan = make_plan(vec![]);
        let result = record_outcome(&plan, 150.0, 1700000000);
        assert_eq!(result.status, PlanStatus::Completed);
        assert!(result.outcome.as_ref().unwrap().profitable);
        assert_eq!(result.outcome.as_ref().unwrap().pnl, 150.0);
    }

    #[test]
    fn test_record_outcome_loss() {
        let plan = make_plan(vec![]);
        let result = record_outcome(&plan, -50.0, 1700000000);
        assert!(!result.outcome.as_ref().unwrap().profitable);
    }

    #[test]
    fn test_compare_op_from_str() {
        assert_eq!(CompareOp::parse("gt"), Some(CompareOp::Gt));
        assert_eq!(CompareOp::parse("lt"), Some(CompareOp::Lt));
        assert_eq!(CompareOp::parse("eq"), Some(CompareOp::Eq));
        assert_eq!(CompareOp::parse("invalid"), None);
    }
}
