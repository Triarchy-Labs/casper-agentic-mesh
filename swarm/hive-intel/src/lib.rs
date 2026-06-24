//! Hive Intel — Collective Intelligence & ML Layer
//!
//! Extracted from Hive-Mind-Engine: ML local inference (<1μs),
//! SIMD-accelerated computations, strategy validation (CPCV),
//! Bayesian memory, regime detection, anomaly detection.
//!
//! This is the "reflexion" layer — it learns from trade outcomes,
//! profiles asset DNA, and provides predictive signals.
pub mod brain;
pub mod orderbook_imbalance;
pub mod ml_local;
pub mod turbo;
pub mod strategy_validator;
pub mod hive_indicators;
pub mod anomaly;
pub mod correlation;
pub mod regime;
pub mod markov;
pub mod reward;
pub mod tilt;
pub mod bloom;
pub mod adaptive;
pub mod affective;
pub mod semantic;
pub mod decay;
pub mod drift;
pub mod replay;
pub mod causal;
pub mod recall;
pub mod hybrid_recall;
pub mod induction;
pub mod patterns;
pub mod entity;
pub mod dqs;
pub mod changepoint;
pub mod risk_sizing;
pub mod legitimacy;
pub mod prospective;
pub mod evolution;
pub mod paper_engine;
pub mod backtester;
pub mod portfolio_guard;
pub mod api;
pub mod hive_engine;
pub mod boost;
pub mod snapshot;
pub mod benchmark;
