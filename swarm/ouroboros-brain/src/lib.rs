//! Ouroboros Brain — LLM Consensus Engine
//!
//! Extracted from Ouroboros-V2: multi-model debate, 15-factor judge,
//! decision memory, circuit breaker, macro guard.
//!
//! This crate provides the AI "brain" — it queries multiple LLM models,
//! runs a bull/bear debate, scores the result through a 15-factor judge,
//! and learns from past decisions via persistent memory.

pub mod openrouter;
pub mod judge;
pub mod config;
pub mod state;
pub mod decision_memory;
pub mod circuit_breaker;
pub mod macro_guard;
pub mod risk_engine;
pub mod ipc;
pub mod hyper;
pub mod agents;
