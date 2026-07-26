#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, Key, URef, U512, CLTyped,
    bytesrepr::{FromBytes, ToBytes, Error as BytesError},
    EntryPoints, EntityEntryPoint, EntryPointAccess, EntryPointType, EntryPointPayment,
    Parameter, account::AccountHash,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const AGENTS_DICT: &str = "agents_dict";
const BOUNTIES_DICT: &str = "bounties_dict";
const ESCROW_PURSE: &str = "escrow_purse";
const MESH_EVENTS_DICT: &str = "mesh_events_dict";
const MESH_EVENT_COUNT: &str = "mesh_event_count";

// ───────────────────────── Data Structures ─────────────────────────

pub struct Agent {
    pub public_key: String,
    pub metadata_uri: String,
    pub rating: u32,
    pub active: bool,
}

impl CLTyped for Agent {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

impl ToBytes for Agent {
    fn serialized_length(&self) -> usize {
        self.public_key.serialized_length()
            + self.metadata_uri.serialized_length()
            + self.rating.serialized_length()
            + self.active.serialized_length()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, BytesError> {
        let mut buffer = Vec::with_capacity(self.serialized_length());
        buffer.extend(self.public_key.to_bytes()?);
        buffer.extend(self.metadata_uri.to_bytes()?);
        buffer.extend(self.rating.to_bytes()?);
        buffer.extend(self.active.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for Agent {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), BytesError> {
        let (public_key, remainder) = String::from_bytes(bytes)?;
        let (metadata_uri, remainder) = String::from_bytes(remainder)?;
        let (rating, remainder) = u32::from_bytes(remainder)?;
        let (active, remainder) = bool::from_bytes(remainder)?;
        let agent = Agent {
            public_key,
            metadata_uri,
            rating,
            active,
        };
        Ok((agent, remainder))
    }
}

pub struct Bounty {
    pub creator: AccountHash,
    pub hunter: AccountHash,
    pub amount: U512,
    pub verifier: AccountHash,
    pub status: u8, // 0 = locked, 1 = released, 2 = refunded
}

impl CLTyped for Bounty {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

impl ToBytes for Bounty {
    fn serialized_length(&self) -> usize {
        self.creator.serialized_length()
            + self.hunter.serialized_length()
            + self.amount.serialized_length()
            + self.verifier.serialized_length()
            + self.status.serialized_length()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, BytesError> {
        let mut buffer = Vec::with_capacity(self.serialized_length());
        buffer.extend(self.creator.to_bytes()?);
        buffer.extend(self.hunter.to_bytes()?);
        buffer.extend(self.amount.to_bytes()?);
        buffer.extend(self.verifier.to_bytes()?);
        buffer.extend(self.status.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for Bounty {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), BytesError> {
        let (creator, remainder) = AccountHash::from_bytes(bytes)?;
        let (hunter, remainder) = AccountHash::from_bytes(remainder)?;
        let (amount, remainder) = U512::from_bytes(remainder)?;
        let (verifier, remainder) = AccountHash::from_bytes(remainder)?;
        let (status, remainder) = u8::from_bytes(remainder)?;
        let bounty = Bounty {
            creator,
            hunter,
            amount,
            verifier,
            status,
        };
        Ok((bounty, remainder))
    }
}

// ───────────────────────── Helpers ─────────────────────────

fn get_dict_uref(name: &str) -> URef {
    runtime::get_key(name)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert()
}

fn get_escrow_purse() -> URef {
    runtime::get_key(ESCROW_PURSE)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert()
}

fn parse_account_hash(s: &str) -> Result<AccountHash, ApiError> {
    if s.starts_with("account-hash-") {
        AccountHash::from_formatted_str(s).map_err(|_| ApiError::User(3))
    } else {
        let formatted = format!("account-hash-{}", s);
        AccountHash::from_formatted_str(&formatted).map_err(|_| ApiError::User(3))
    }
}

fn emit_mesh_event(event_name: &str, payload: String) {
    let uref = runtime::get_key(MESH_EVENT_COUNT)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();
    let current: u64 = storage::read(uref).unwrap_or_revert().unwrap_or(0);
    let next = current.saturating_add(1);
    storage::write(uref, next);

    let dict = get_dict_uref(MESH_EVENTS_DICT);
    let record = format!("event={};payload={}", event_name, payload);
    storage::dictionary_put(dict, &current.to_string(), record);
}

// ───────────────────────── Entry Points ─────────────────────────

/// Minimal no-op entry point to verify WASM execution works at all.
#[no_mangle]
pub extern "C" fn ping() {}

/// Called once after contract installation to initialize dictionaries and escrow purse.
#[no_mangle]
pub extern "C" fn init() {
    // Guard: prevent re-initialization
    if runtime::get_key(AGENTS_DICT).is_some() {
        runtime::revert(ApiError::User(0)); // AlreadyInitialized
    }

    // Create dictionaries in contract context
    storage::new_dictionary(AGENTS_DICT).unwrap_or_revert();
    storage::new_dictionary(BOUNTIES_DICT).unwrap_or_revert();
    storage::new_dictionary(MESH_EVENTS_DICT).unwrap_or_revert();

    let counter = storage::new_uref(0u64);
    runtime::put_key(MESH_EVENT_COUNT, Key::from(counter));

    // Create escrow purse owned by the contract
    let escrow_purse = system::create_purse();
    runtime::put_key(ESCROW_PURSE, Key::from(escrow_purse));
}

#[no_mangle]
pub extern "C" fn register_agent() {
    let public_key: String = runtime::get_named_arg("public_key");
    let metadata_uri: String = runtime::get_named_arg("metadata_uri");

    let dict_uref = get_dict_uref(AGENTS_DICT);

    let agent = Agent {
        public_key: public_key.clone(),
        metadata_uri,
        rating: 0,
        active: true,
    };

    let caller_str = runtime::get_caller().to_string();
    storage::dictionary_put(dict_uref, &caller_str, agent);

    emit_mesh_event("REGISTER_AGENT", format!("caller={caller_str};pubkey={public_key}"));
}

#[no_mangle]
pub extern "C" fn create_bounty() {
    let task_id: String = runtime::get_named_arg("task_id");
    let hunter_str: String = runtime::get_named_arg("hunter");
    let amount: U512 = runtime::get_named_arg("amount");
    let verifier_str: String = runtime::get_named_arg("verifier");
    let source_purse: URef = runtime::get_named_arg("source_purse");

    // Zero-value math check
    if amount == U512::zero() {
        runtime::revert(ApiError::User(5)); // ZeroDeposit
    }

    let dict_uref = get_dict_uref(BOUNTIES_DICT);

    // Check if task_id already exists (Prevent overwrite / locked fund abandonment)
    let existing: Option<Bounty> = storage::dictionary_get(dict_uref, &task_id).unwrap_or_revert();
    if existing.is_some() {
        runtime::revert(ApiError::User(4)); // TaskIdAlreadyExists
    }

    let creator = runtime::get_caller();
    let hunter = parse_account_hash(&hunter_str).unwrap_or_revert();
    let verifier = parse_account_hash(&verifier_str).unwrap_or_revert();

    let escrow_purse = get_escrow_purse();

    // Transfer from caller's purse to the contract's escrow purse
    system::transfer_from_purse_to_purse(source_purse, escrow_purse, amount, None)
        .unwrap_or_revert();

    let bounty = Bounty {
        creator,
        hunter,
        amount,
        verifier,
        status: 0,
    };

    storage::dictionary_put(dict_uref, &task_id, bounty);

    emit_mesh_event("CREATE_BOUNTY", format!("task_id={task_id};amount={amount}"));
}

#[no_mangle]
pub extern "C" fn release_bounty() {
    let task_id: String = runtime::get_named_arg("task_id");

    let dict_uref = get_dict_uref(BOUNTIES_DICT);
    let mut bounty: Bounty = storage::dictionary_get(dict_uref, &task_id)
        .unwrap_or_revert()
        .unwrap_or_revert();

    let caller = runtime::get_caller();
    if caller != bounty.verifier {
        runtime::revert(ApiError::User(1)); // Unauthorized
    }

    if bounty.status != 0 {
        runtime::revert(ApiError::User(2)); // Not locked
    }

    // CEI: Update contract state BEFORE external interactions
    bounty.status = 1; // Completed / Released
    storage::dictionary_put(dict_uref, &task_id, &bounty);

    let escrow_purse = get_escrow_purse();
    system::transfer_from_purse_to_account(escrow_purse, bounty.hunter, bounty.amount, None)
        .unwrap_or_revert();

    emit_mesh_event("RELEASE_BOUNTY", format!("task_id={task_id};hunter={}", bounty.hunter));
}

#[no_mangle]
pub extern "C" fn refund_bounty() {
    let task_id: String = runtime::get_named_arg("task_id");

    let dict_uref = get_dict_uref(BOUNTIES_DICT);
    let mut bounty: Bounty = storage::dictionary_get(dict_uref, &task_id)
        .unwrap_or_revert()
        .unwrap_or_revert();

    let caller = runtime::get_caller();
    if caller != bounty.verifier && caller != bounty.creator {
        runtime::revert(ApiError::User(1)); // Unauthorized
    }

    if bounty.status != 0 {
        runtime::revert(ApiError::User(2)); // Not locked
    }

    // CEI: Update contract state BEFORE external interactions
    bounty.status = 2; // Refunded
    storage::dictionary_put(dict_uref, &task_id, &bounty);

    let escrow_purse = get_escrow_purse();
    system::transfer_from_purse_to_account(escrow_purse, bounty.creator, bounty.amount, None)
        .unwrap_or_revert();

    emit_mesh_event("REFUND_BOUNTY", format!("task_id={task_id};creator={}", bounty.creator));
}

// ───────────────────────── Installation ─────────────────────────

fn build_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntityEntryPoint::new(
        "ping",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points.add_entry_point(EntityEntryPoint::new(
        "init",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points.add_entry_point(EntityEntryPoint::new(
        "register_agent",
        vec![
            Parameter::new("public_key", CLType::String),
            Parameter::new("metadata_uri", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points.add_entry_point(EntityEntryPoint::new(
        "create_bounty",
        vec![
            Parameter::new("task_id", CLType::String),
            Parameter::new("hunter", CLType::String),
            Parameter::new("amount", CLType::U512),
            Parameter::new("verifier", CLType::String),
            Parameter::new("source_purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points.add_entry_point(EntityEntryPoint::new(
        "release_bounty",
        vec![
            Parameter::new("task_id", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points.add_entry_point(EntityEntryPoint::new(
        "refund_bounty",
        vec![
            Parameter::new("task_id", CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points
}

/// Session-level installer. Creates the contract package + version.
#[no_mangle]
pub extern "C" fn call() {
    let entry_points = build_entry_points();

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        None,
        Some("casper_agentic_mesh_package".to_string()),
        Some("casper_agentic_mesh_access".to_string()),
        None,
    );

    runtime::put_key("casper_agentic_mesh_contract", Key::from(contract_hash));
}

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
