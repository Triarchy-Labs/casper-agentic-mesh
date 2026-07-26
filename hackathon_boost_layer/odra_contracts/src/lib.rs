#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]
#![allow(unexpected_cfgs)]

extern crate alloc;
use alloc::vec::Vec;
use odra::prelude::*;
use odra::{Address, Mapping, UnwrapOrRevert, Var};

pub type AssetId = [u8; 32];

#[derive(odra::OdraType, Debug)]
pub struct PriceData {
    pub price: u64,
    pub last_updated: u64,
    pub decimals: u8,
}

#[derive(odra::OdraError, Debug)]
pub enum Error {
    Unauthorized = 1,
    NoPendingAdmin = 2,
    BatchTooLarge = 3,
    LengthMismatch = 4,
}

#[odra::module(events = [PriceUpdated, PricesUpdated, OwnershipProposed, OwnershipTransferred])]
pub struct RwaOracle {
    pub prices: Mapping<AssetId, PriceData>,
    pub admin: Var<Address>,
    pub pending_admin: Var<Address>,
}

#[odra::module]
impl RwaOracle {
    #[odra(init)]
    pub fn init(&mut self) {
        self.admin.set(self.env().caller());
        // Do not explicitly set pending_admin to None, uninitialized Vars in Odra are None by default
        // thereby saving dictionary gas on initialization.
    }

    fn assert_admin(&self) {
        let current_admin = self
            .admin
            .get()
            .unwrap_or_revert_with(&self.env(), Error::Unauthorized);

        if self.env().caller() != current_admin {
            self.env().revert(Error::Unauthorized);
        }
    }

    pub fn set_price(&mut self, asset: AssetId, price: u64, decimals: u8) {
        self.assert_admin();

        let now = self.env().get_block_time();

        self.prices.set(
            &asset,
            PriceData {
                price,
                last_updated: now,
                decimals,
            },
        );

        self.env().emit_event(PriceUpdated {
            asset,
            price,
            last_updated: now,
            decimals,
        });
    }

    pub fn set_prices(&mut self, assets: Vec<AssetId>, prices: Vec<u64>, decimals: Vec<u8>) {
        self.assert_admin();

        if assets.len() > 50 {
            self.env().revert(Error::BatchTooLarge);
        }

        if assets.len() != prices.len() || assets.len() != decimals.len() {
            self.env().revert(Error::LengthMismatch);
        }

        let now = self.env().get_block_time();

        for ((asset, &price), &decimal) in assets.iter().zip(prices.iter()).zip(decimals.iter()) {
            self.prices.set(
                asset,
                PriceData {
                    price,
                    last_updated: now,
                    decimals: decimal,
                },
            );
        }

        self.env().emit_event(PricesUpdated {
            assets,
            prices,
            last_updated: now,
            decimals,
        });
    }

    pub fn get_price_data(&self, asset: AssetId) -> Option<PriceData> {
        self.prices.get(&asset)
    }

    // Two-step ownership transfer
    pub fn propose_ownership(&mut self, new_admin: Address) {
        self.assert_admin();

        self.pending_admin.set(new_admin);

        self.env().emit_event(OwnershipProposed {
            proposed_admin: new_admin,
        });
    }

    pub fn accept_ownership(&mut self) {
        let pending = self
            .pending_admin
            .get()
            .unwrap_or_revert_with(&self.env(), Error::NoPendingAdmin);

        if self.env().caller() != pending {
            self.env().revert(Error::Unauthorized);
        }

        let _old_admin = self.admin.get().unwrap();
        self.admin.set(pending);

        // In Odra 0.8, there is no direct var.clear(), but ownership transferred is now finalized
        self.env().emit_event(OwnershipTransferred {
            previous_admin: _old_admin,
            new_admin: pending,
        });
    }
}

#[derive(odra::Event)]
pub struct PriceUpdated {
    pub asset: AssetId,
    pub price: u64,
    pub last_updated: u64,
    pub decimals: u8,
}

#[derive(odra::Event)]
pub struct PricesUpdated {
    pub assets: Vec<AssetId>,
    pub prices: Vec<u64>,
    pub last_updated: u64,
    pub decimals: Vec<u8>,
}

#[derive(odra::Event)]
pub struct OwnershipProposed {
    pub proposed_admin: Address,
}

#[derive(odra::Event)]
pub struct OwnershipTransferred {
    pub previous_admin: Address,
    pub new_admin: Address,
}
