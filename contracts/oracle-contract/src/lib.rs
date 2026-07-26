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
use alloc::vec::Vec;
use alloc::format;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, Key, U512, CLTyped,
    bytesrepr::{FromBytes, ToBytes, Error as BytesreprError},
    EntryPoints, EntityEntryPoint, EntryPointAccess, EntryPointType, EntryPointPayment,
    Parameter, URef,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ORACLES_DICT: &str = "oracles_dict";       // agent identity:   caller (account_hash str) -> OracleRecord
const READINGS_DICT: &str = "readings_dict";     // latest reading:   asset -> ReadingRecord
const REPUTATION_DICT: &str = "reputation_dict"; // accuracy/score:   caller (account_hash str) -> u64
const EVENTS_DICT: &str = "events_dict";         // append-only log:  seq(string) -> EventRecord
const EVENT_COUNT: &str = "event_count";         // monotonic counter (uref<u64>)

// ───────────────────────── Data Structures ─────────────────────────

pub struct OracleRecord {
    pub public_key: String,
    pub metadata_uri: String,
    pub registered_at: u64,
}

impl CLTyped for OracleRecord {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

impl ToBytes for OracleRecord {
    fn serialized_length(&self) -> usize {
        self.public_key.serialized_length()
            + self.metadata_uri.serialized_length()
            + self.registered_at.serialized_length()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, BytesreprError> {
        let mut buffer = Vec::with_capacity(self.serialized_length());
        buffer.extend(self.public_key.to_bytes()?);
        buffer.extend(self.metadata_uri.to_bytes()?);
        buffer.extend(self.registered_at.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for OracleRecord {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), BytesreprError> {
        let (public_key, remainder) = String::from_bytes(bytes)?;
        let (metadata_uri, remainder) = String::from_bytes(remainder)?;
        let (registered_at, remainder) = u64::from_bytes(remainder)?;
        Ok((
            OracleRecord {
                public_key,
                metadata_uri,
                registered_at,
            },
            remainder,
        ))
    }
}

pub struct ReadingRecord {
    pub value: U512,
    pub by: String,
    pub seq: u64,
    pub timestamp: u64,
}

impl CLTyped for ReadingRecord {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

impl ToBytes for ReadingRecord {
    fn serialized_length(&self) -> usize {
        self.value.serialized_length()
            + self.by.serialized_length()
            + self.seq.serialized_length()
            + self.timestamp.serialized_length()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, BytesreprError> {
        let mut buffer = Vec::with_capacity(self.serialized_length());
        buffer.extend(self.value.to_bytes()?);
        buffer.extend(self.by.to_bytes()?);
        buffer.extend(self.seq.to_bytes()?);
        buffer.extend(self.timestamp.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for ReadingRecord {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), BytesreprError> {
        let (value, remainder) = U512::from_bytes(bytes)?;
        let (by, remainder) = String::from_bytes(remainder)?;
        let (seq, remainder) = u64::from_bytes(remainder)?;
        let (timestamp, remainder) = u64::from_bytes(remainder)?;
        Ok((
            ReadingRecord {
                value,
                by,
                seq,
                timestamp,
            },
            remainder,
        ))
    }
}

pub struct EventRecord {
    pub event_type: String,
    pub payload: String,
    pub timestamp: u64,
}

impl CLTyped for EventRecord {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

impl ToBytes for EventRecord {
    fn serialized_length(&self) -> usize {
        self.event_type.serialized_length()
            + self.payload.serialized_length()
            + self.timestamp.serialized_length()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, BytesreprError> {
        let mut buffer = Vec::with_capacity(self.serialized_length());
        buffer.extend(self.event_type.to_bytes()?);
        buffer.extend(self.payload.to_bytes()?);
        buffer.extend(self.timestamp.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for EventRecord {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), BytesreprError> {
        let (event_type, remainder) = String::from_bytes(bytes)?;
        let (payload, remainder) = String::from_bytes(remainder)?;
        let (timestamp, remainder) = u64::from_bytes(remainder)?;
        Ok((
            EventRecord {
                event_type,
                payload,
                timestamp,
            },
            remainder,
        ))
    }
}

// ───────────────────────── Helpers ─────────────────────────

fn dict_uref(name: &str) -> URef {
    let key = runtime::get_key(name).unwrap_or_revert_with(ApiError::User(20));
    key.into_uref().unwrap_or_revert_with(ApiError::User(21))
}

fn next_event_seq() -> u64 {
    let uref = runtime::get_key(EVENT_COUNT)
        .unwrap_or_revert_with(ApiError::User(22))
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(23));
    let current: u64 = storage::read(uref).unwrap_or_revert().unwrap_or(0);
    let next = current.checked_add(1).unwrap_or_revert_with(ApiError::User(24));
    storage::write(uref, next);
    current
}

fn emit(event_type: &str, payload: String) {
    let seq = next_event_seq();
    let events = dict_uref(EVENTS_DICT);
    let now = runtime::get_blocktime().into();
    let record = EventRecord {
        event_type: event_type.to_string(),
        payload,
        timestamp: now,
    };
    storage::dictionary_put(events, &seq.to_string(), record);
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
    let caller = runtime::get_caller().to_string();

    let oracles = dict_uref(ORACLES_DICT);
    let record = OracleRecord {
        public_key: public_key.clone(),
        metadata_uri,
        registered_at: runtime::get_blocktime().into(),
    };
    // Store identity record mapped to caller account hash
    storage::dictionary_put(oracles, &caller, record);

    // Seed reputation entry if absent.
    let reputation = dict_uref(REPUTATION_DICT);
    let existing: Option<u64> = storage::dictionary_get(reputation, &caller).unwrap_or_revert();
    if existing.is_none() {
        storage::dictionary_put(reputation, &caller, 0u64);
    }

    emit("REGISTER", format!("agent={caller};key={public_key}"));
}

/// Post a real-world asset reading on-chain and accrue reputation.
#[no_mangle]
pub extern "C" fn post_reading() {
    let asset: String = runtime::get_named_arg("asset");
    let value: U512 = runtime::get_named_arg("value");
    let caller = runtime::get_caller().to_string();

    // Verify identity: caller MUST be a registered oracle
    let oracles = dict_uref(ORACLES_DICT);
    let oracle_record: Option<OracleRecord> = storage::dictionary_get(oracles, &caller).unwrap_or_revert();
    if oracle_record.is_none() {
        runtime::revert(ApiError::User(30)); // UnauthorizedOracle
    }

    let seq = next_event_seq();
    let now = runtime::get_blocktime().into();

    // Store the latest reading for this asset with structured binary record
    let readings = dict_uref(READINGS_DICT);
    let record = ReadingRecord {
        value,
        by: caller.clone(),
        seq,
        timestamp: now,
    };
    storage::dictionary_put(readings, &asset, record);

    // Accrue reputation for the reporting agent safely
    let reputation = dict_uref(REPUTATION_DICT);
    let score: u64 = storage::dictionary_get(reputation, &caller)
        .unwrap_or_revert()
        .unwrap_or(0);
    let new_score = score.saturating_add(1);
    storage::dictionary_put(reputation, &caller, new_score);

    emit("READING", format!("asset={asset};value={value};by={caller}"));
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

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
