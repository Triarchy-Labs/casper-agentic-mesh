//! Mantle Chain — On-Chain Adapter for Mantle L2
//!
//! Provides Alloy-based RPC client, ERC-8004 registry interaction,
//! DEX execution (Agni Finance), and on-chain event logging.
//!
//! Already deployed contracts on Mantle Mainnet (chain 5000):
//!   ERC8004Registry: 0x1150f09ae885e6E7BcC0cb38feDd200d7f580008
//!   X402FlashLiquidator: 0x30daC056a87D5844Fb5BE47Fb5412A6Bee83072d
//!   Agent #1 NFT: Token ID 1
//!   Wallet: 0xF02332A7d92C86631Ea30d49D9778994B9277c79

pub mod provider;
pub mod erc8004;
pub mod onchain;
pub mod wallet;
pub mod dex;
