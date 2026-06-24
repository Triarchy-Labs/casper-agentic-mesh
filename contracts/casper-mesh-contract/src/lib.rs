#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, Key, URef, U512, CLTyped, bytesrepr::{FromBytes, ToBytes, Error},
    EntryPoints, EntityEntryPoint, EntryPointAccess, EntryPointType, EntryPointPayment,
    Parameter, NamedKeys,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const AGENTS_DICT: &str = "agents_dict";
const BOUNTIES_DICT: &str = "bounties_dict";
const ESCROW_PURSE: &str = "escrow_purse";

// ───────────────────────── Data Structures ─────────────────────────

pub struct Agent {
    public_key: String,
    metadata_uri: String,
    rating: u32,
    active: bool,
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

    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut buffer = alloc::vec::Vec::with_capacity(self.serialized_length());
        buffer.extend(self.public_key.to_bytes()?);
        buffer.extend(self.metadata_uri.to_bytes()?);
        buffer.extend(self.rating.to_bytes()?);
        buffer.extend(self.active.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for Agent {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
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
    creator: String,
    hunter: String,
    amount: U512,
    verifier: String,
    status: u8, // 0 = locked, 1 = released, 2 = refunded
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

    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut buffer = alloc::vec::Vec::with_capacity(self.serialized_length());
        buffer.extend(self.creator.to_bytes()?);
        buffer.extend(self.hunter.to_bytes()?);
        buffer.extend(self.amount.to_bytes()?);
        buffer.extend(self.verifier.to_bytes()?);
        buffer.extend(self.status.to_bytes()?);
        Ok(buffer)
    }
}

impl FromBytes for Bounty {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (creator, remainder) = String::from_bytes(bytes)?;
        let (hunter, remainder) = String::from_bytes(remainder)?;
        let (amount, remainder) = U512::from_bytes(remainder)?;
        let (verifier, remainder) = String::from_bytes(remainder)?;
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
    let key_opt = runtime::get_key(name);
    if key_opt.is_none() {
        runtime::revert(ApiError::User(20)); // Dict not found
    }
    let uref_opt = key_opt.unwrap_or_revert().into_uref();
    if uref_opt.is_none() {
        runtime::revert(ApiError::User(21)); // Key is not a URef
    }
    match uref_opt {
        Some(uref) => uref,
        None => runtime::revert(ApiError::User(21)), // Key is not a URef
    }
}

fn get_escrow_purse() -> URef {
    let key_opt = runtime::get_key(ESCROW_PURSE);
    if key_opt.is_none() {
        runtime::revert(ApiError::User(22)); // Escrow purse not found
    }
    let uref_opt = key_opt.unwrap_or_revert().into_uref();
    if uref_opt.is_none() {
        runtime::revert(ApiError::User(23)); // Key is not a URef
    }
    match uref_opt {
        Some(uref) => uref,
        None => runtime::revert(ApiError::User(23)), // Key is not a URef
    }
}

// ───────────────────────── Entry Points ─────────────────────────

/// Minimal no-op entry point to verify WASM execution works at all.
#[no_mangle]
pub extern "C" fn ping() {
    // Intentionally empty — just confirms the VM can call this contract
}

/// Called once after contract installation to initialize dictionaries and escrow purse.
/// This runs in CONTRACT context (not session), which is required for Casper 2.0.
#[no_mangle]
pub extern "C" fn init() {
    // Guard: prevent re-initialization
    if runtime::get_key(AGENTS_DICT).is_some() {
        runtime::revert(ApiError::User(0)); // AlreadyInitialized
    }

    // Create dictionaries in contract context
    storage::new_dictionary(AGENTS_DICT).unwrap_or_revert();
    storage::new_dictionary(BOUNTIES_DICT).unwrap_or_revert();

    // Create escrow purse owned by the contract
    let escrow_purse = system::create_purse();
    runtime::put_key(ESCROW_PURSE, Key::from(escrow_purse));
}

#[no_mangle]
pub extern "C" fn register_agent() {
    let public_key: String = runtime::get_named_arg("public_key");
    let metadata_uri: String = runtime::get_named_arg("metadata_uri");

    // Step 1: Get the key
    let key_opt = runtime::get_key(AGENTS_DICT);
    if key_opt.is_none() {
        runtime::revert(ApiError::User(10)); // AGENTS_DICT not found
    }
    let key = key_opt.unwrap_or_revert();

    // Step 2: Try to convert to URef
    let uref_opt = key.into_uref();
    if uref_opt.is_none() {
        runtime::revert(ApiError::User(11)); // Key is not a URef
    }
    let dict_uref = uref_opt.unwrap_or_revert();

    // Step 3: Store a simple string first (not the custom Agent struct)
    storage::dictionary_put(dict_uref, &public_key, metadata_uri);
}

#[no_mangle]
pub extern "C" fn create_bounty() {
    let task_id: String = runtime::get_named_arg("task_id");
    let hunter: String = runtime::get_named_arg("hunter");
    let amount: U512 = runtime::get_named_arg("amount");
    let verifier: String = runtime::get_named_arg("verifier");
    let source_purse: URef = runtime::get_named_arg("source_purse");

    let dict_uref = get_dict_uref(BOUNTIES_DICT);
    let escrow_purse = get_escrow_purse();

    // Transfer from caller's purse to the contract's escrow purse
    if system::transfer_from_purse_to_purse(source_purse, escrow_purse, amount, None).is_err() {
        runtime::revert(ApiError::User(30)); // Transfer failed
    }

    let bounty = Bounty {
        creator: runtime::get_caller().to_string(),
        hunter,
        amount,
        verifier,
        status: 0,
    };

    storage::dictionary_put(dict_uref, &task_id, bounty);
}

#[no_mangle]
pub extern "C" fn release_bounty() {
    let task_id: String = runtime::get_named_arg("task_id");
    let target_purse: URef = runtime::get_named_arg("target_purse");

    let dict_uref = get_dict_uref(BOUNTIES_DICT);
    let bounty_opt: Option<Bounty> = match storage::dictionary_get(dict_uref, &task_id) {
        Ok(opt) => opt,
        Err(_) => runtime::revert(ApiError::User(40)), // Failed to read dictionary
    };
    if bounty_opt.is_none() {
        runtime::revert(ApiError::User(41)); // Bounty not found
    }
    let mut bounty = bounty_opt.unwrap();

    let caller = runtime::get_caller().to_string();
    if caller != bounty.verifier {
        runtime::revert(ApiError::User(1)); // Unauthorized
    }

    if bounty.status != 0 {
        runtime::revert(ApiError::User(2)); // Not locked
    }

    let escrow_purse = get_escrow_purse();
    if system::transfer_from_purse_to_purse(escrow_purse, target_purse, bounty.amount, None).is_err() {
        runtime::revert(ApiError::User(30)); // Transfer failed
    }

    bounty.status = 1; // Completed
    storage::dictionary_put(dict_uref, &task_id, bounty);
}

#[no_mangle]
pub extern "C" fn refund_bounty() {
    let task_id: String = runtime::get_named_arg("task_id");
    let target_purse: URef = runtime::get_named_arg("target_purse");

    let dict_uref = get_dict_uref(BOUNTIES_DICT);
    let bounty_opt: Option<Bounty> = match storage::dictionary_get(dict_uref, &task_id) {
        Ok(opt) => opt,
        Err(_) => runtime::revert(ApiError::User(40)), // Failed to read dictionary
    };
    if bounty_opt.is_none() {
        runtime::revert(ApiError::User(41)); // Bounty not found
    }
    let mut bounty = bounty_opt.unwrap();

    let caller = runtime::get_caller().to_string();
    if caller != bounty.verifier && caller != bounty.creator {
        runtime::revert(ApiError::User(1)); // Unauthorized
    }

    if bounty.status != 0 {
        runtime::revert(ApiError::User(2)); // Not locked
    }

    let escrow_purse = get_escrow_purse();
    if system::transfer_from_purse_to_purse(escrow_purse, target_purse, bounty.amount, None).is_err() {
        runtime::revert(ApiError::User(30)); // Transfer failed
    }

    bounty.status = 2; // Refunded
    storage::dictionary_put(dict_uref, &task_id, bounty);
}

// ───────────────────────── Installation ─────────────────────────

fn build_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    // ping: minimal no-op for VM diagnostics
    entry_points.add_entry_point(EntityEntryPoint::new(
        "ping",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    // init: one-time state setup (dictionaries + escrow purse)
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
            Parameter::new("target_purse", CLType::URef),
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
            Parameter::new("target_purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Called,
        EntryPointPayment::Caller,
    ));

    entry_points
}

/// Session-level installer. Creates the contract package + version,
/// then immediately calls `init()` to set up state IN the contract context.
#[no_mangle]
pub extern "C" fn call() {
    let entry_points = build_entry_points();

    // No named_keys passed — state is created by `init()` in contract context
    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        None, // ← empty named_keys; init() creates them in contract context
        Some("casper_agentic_mesh_package_v2".to_string()),
        Some("casper_agentic_mesh_access_v2".to_string()),
        None,
    );

    // Store the contract hash for external reference
    runtime::put_key("casper_agentic_mesh_contract_v2", Key::from(contract_hash));

    // NOTE: init() must be called as a SEPARATE transaction after deployment.
    // runtime::call_contract from session context is not reliable in Casper 2.0.
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
