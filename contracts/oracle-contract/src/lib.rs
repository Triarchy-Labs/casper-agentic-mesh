#![no_std]
#![no_main]

//! RWA Oracle Contract — verifiable on-chain identity, data feed and reputation.
//!
//! Implements buildathon example direction #2 ("RWA Oracle Agents with verifiable
//! on-chain identity"): autonomous agents register an identity, post real-world
//! asset readings on-chain, and accrue a reputation score per accepted reading.
//! Every state change appends to an on-chain event log readable by indexers.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::format;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, Key, U512,
    EntryPoints, EntityEntryPoint, EntryPointAccess, EntryPointType, EntryPointPayment,
    Parameter,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ORACLES_DICT: &str = "oracles_dict";       // agent identity:   pubkey -> metadata
const READINGS_DICT: &str = "readings_dict";     // latest reading:   asset  -> "value=..;by=..;seq=.."
const REPUTATION_DICT: &str = "reputation_dict"; // accuracy/score:   account_hash -> u64
const EVENTS_DICT: &str = "events_dict";         // append-only log:  seq(string) -> event string
const EVENT_COUNT: &str = "event_count";         // monotonic counter (uref<u64>)

fn dict_uref(name: &str) -> casper_types::URef {
    let key = runtime::get_key(name).unwrap_or_revert_with(ApiError::User(20));
    key.into_uref().unwrap_or_revert_with(ApiError::User(21))
}

fn next_event_seq() -> u64 {
    let uref = runtime::get_key(EVENT_COUNT)
        .unwrap_or_revert_with(ApiError::User(22))
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(23));
    let current: u64 = storage::read(uref).unwrap_or_revert().unwrap_or(0);
    storage::write(uref, current + 1);
    current
}

fn emit(event: String) {
    let seq = next_event_seq();
    let events = dict_uref(EVENTS_DICT);
    storage::dictionary_put(events, &seq.to_string(), event);
}

#[no_mangle]
pub extern "C" fn ping() {}

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(ORACLES_DICT).is_some() {
        runtime::revert(ApiError::User(0)); // AlreadyInitialized
    }
    storage::new_dictionary(ORACLES_DICT).unwrap_or_revert();
    storage::new_dictionary(READINGS_DICT).unwrap_or_revert();
    storage::new_dictionary(REPUTATION_DICT).unwrap_or_revert();
    storage::new_dictionary(EVENTS_DICT).unwrap_or_revert();
    let counter = storage::new_uref(0u64);
    runtime::put_key(EVENT_COUNT, Key::from(counter));
}

/// Register an oracle agent's verifiable identity.
#[no_mangle]
pub extern "C" fn register_oracle() {
    let public_key: String = runtime::get_named_arg("public_key");
    let metadata_uri: String = runtime::get_named_arg("metadata_uri");

    let oracles = dict_uref(ORACLES_DICT);
    storage::dictionary_put(oracles, &public_key, metadata_uri);

    // Seed reputation entry if absent.
    let caller = runtime::get_caller().to_string();
    let reputation = dict_uref(REPUTATION_DICT);
    let existing: Option<u64> = storage::dictionary_get(reputation, &caller).unwrap_or_revert();
    if existing.is_none() {
        storage::dictionary_put(reputation, &caller, 0u64);
    }

    emit(format!("REGISTER;agent={caller};key={public_key}"));
}

/// Post a real-world asset reading on-chain and accrue reputation.
#[no_mangle]
pub extern "C" fn post_reading() {
    let asset: String = runtime::get_named_arg("asset");
    let value: U512 = runtime::get_named_arg("value");

    let caller = runtime::get_caller().to_string();
    let seq = next_event_seq();

    // Store the latest reading for this asset.
    let readings = dict_uref(READINGS_DICT);
    let record = format!("value={};by={};seq={}", value, caller, seq);
    storage::dictionary_put(readings, &asset, record);

    // Accrue reputation for the reporting agent.
    let reputation = dict_uref(REPUTATION_DICT);
    let score: u64 = storage::dictionary_get(reputation, &caller)
        .unwrap_or_revert()
        .unwrap_or(0);
    storage::dictionary_put(reputation, &caller, score + 1);

    emit(format!("READING;asset={asset};value={value};by={caller}"));
}

// ───────────────────────── Installation ─────────────────────────

fn build_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntityEntryPoint::new(
        "ping", vec![], CLType::Unit,
        EntryPointAccess::Public, EntryPointType::Called, EntryPointPayment::Caller,
    ));
    entry_points.add_entry_point(EntityEntryPoint::new(
        "init", vec![], CLType::Unit,
        EntryPointAccess::Public, EntryPointType::Called, EntryPointPayment::Caller,
    ));
    entry_points.add_entry_point(EntityEntryPoint::new(
        "register_oracle",
        vec![
            Parameter::new("public_key", CLType::String),
            Parameter::new("metadata_uri", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public, EntryPointType::Called, EntryPointPayment::Caller,
    ));
    entry_points.add_entry_point(EntityEntryPoint::new(
        "post_reading",
        vec![
            Parameter::new("asset", CLType::String),
            Parameter::new("value", CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public, EntryPointType::Called, EntryPointPayment::Caller,
    ));

    entry_points
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = build_entry_points();
    let (contract_hash, _version) = storage::new_contract(
        entry_points,
        None,
        Some("triarchy_oracle_package".to_string()),
        Some("triarchy_oracle_access".to_string()),
        None,
    );
    runtime::put_key("triarchy_oracle_contract", Key::from(contract_hash));
    // init() is called as a separate tx after deployment (Casper 2.0 session rule).
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
