#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::ContractPackageHash, runtime_args, ApiError, RuntimeArgs, URef, U512,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Session code executed in the *caller's account context*.
///
/// A stored contract cannot withdraw from an account's main purse — Casper
/// denies access to the main purse inside contract context. The canonical
/// pattern is to run session code here, where `account::get_main_purse()` is
/// accessible, create a fresh temp purse, fund it from the main purse, then
/// hand that temp purse to the stored contract's `create_bounty` entry point.
#[no_mangle]
pub extern "C" fn call() {
    let package_str: String = runtime::get_named_arg("contract_package");
    let amount: U512 = runtime::get_named_arg("amount");
    let task_id: String = runtime::get_named_arg("task_id");
    let hunter: String = runtime::get_named_arg("hunter");
    let verifier: String = runtime::get_named_arg("verifier");

    let package_hash: ContractPackageHash =
        match ContractPackageHash::from_formatted_str(&package_str) {
            Ok(h) => h,
            Err(_) => runtime::revert(ApiError::User(50)),
        };

    // Fund a fresh temp purse from the account main purse (allowed in session ctx).
    let temp_purse: URef = system::create_purse();
    system::transfer_from_purse_to_purse(account::get_main_purse(), temp_purse, amount, None)
        .unwrap_or_revert_with(ApiError::User(51));

    // Call the stored contract, passing the funded temp purse it CAN access.
    runtime::call_versioned_contract::<()>(
        package_hash,
        None,
        "create_bounty",
        runtime_args! {
            "task_id" => task_id,
            "hunter" => hunter,
            "amount" => amount,
            "verifier" => verifier,
            "source_purse" => temp_purse,
        },
    );
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
