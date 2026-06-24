use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use std::error::Error;
use clap::{Parser, Subcommand};
use reqwest::Client;

#[derive(Parser, Clone, Debug)]
#[command(name = "casper-client")]
#[command(about = "Rust JSON-RPC Client for Casper Network with transaction execution", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "https://node.testnet.casper.network/rpc")]
    node: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Get current chain state root hash
    StateRootHash,

    /// Get details of a deploy/transaction by hash
    GetDeploy {
        #[arg(help = "The deploy or transaction hash (hex)")]
        hash: String,
    },

    /// Get main purse URef of an account hash
    GetAccountPurse {
        #[arg(help = "Account hash hex (with or without 'account-hash-' prefix)")]
        account: String,
    },

    /// Get balance of a purse by its URef
    GetBalance {
        #[arg(help = "Purse URef (e.g. uref-abc...-007)")]
        uref: String,
    },

    /// Query a dictionary item from contract state
    QueryDictionary {
        #[arg(long, help = "Contract hash (hex, with or without 'hash-' or 'contract-' prefix)")]
        contract: String,
        #[arg(long, help = "Name of the dictionary in the contract")]
        dict_name: String,
        #[arg(long, help = "Key of the dictionary item")]
        key: String,
    },

    /// Deploy a compiled smart contract WASM
    DeployWasm {
        #[arg(long, help = "Path to compiled .wasm contract binary")]
        wasm: String,
        #[arg(long, help = "Path to secret_key.pem")]
        secret_key: String,
        #[arg(long, default_value = "ed25519", help = "Key algorithm (ed25519 or secp256k1)")]
        algo: String,
        #[arg(long, default_value = "casper-test", help = "Casper network chain name")]
        chain: String,
        #[arg(long, default_value = "200000000000", help = "Gas payment amount in motes (default: 200 CSPR)")]
        payment: String,
    },

    /// Call a contract entrypoint
    CallContract {
        #[arg(long, help = "Contract hash (hex)")]
        contract: String,
        #[arg(long, help = "Name of the entrypoint to call")]
        entrypoint: String,
        #[arg(long, default_value = "", help = "Arguments in key:type:val format, comma-separated (e.g. 'receiver:key:account-hash-123,amount:u512:100')")]
        args: String,
        #[arg(long, help = "Path to secret_key.pem")]
        secret_key: String,
        #[arg(long, default_value = "ed25519", help = "Key algorithm (ed25519 or secp256k1)")]
        algo: String,
        #[arg(long, default_value = "casper-test", help = "Casper network chain name")]
        chain: String,
        #[arg(long, default_value = "5000000000", help = "Gas payment amount in motes")]
        payment: String,
    },
}

#[derive(Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: serde_json::Value,
}

impl RpcRequest {
    fn new(method: &'static str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        }
    }
}

async fn post_rpc(client: &Client, node: &str, req: RpcRequest) -> Result<serde_json::Value, Box<dyn Error>> {
    let resp = client.post(node)
        .json(&req)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("HTTP request failed with status: {}", resp.status()).into());
    }

    let json_val: serde_json::Value = resp.json().await?;
    if let Some(err) = json_val.get("error") {
        return Err(format!("RPC error: {}", err.to_string()).into());
    }

    Ok(json_val)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::StateRootHash => {
            let req = RpcRequest::new("chain_get_state_root_hash", json!([]));
            let res = post_rpc(&client, &cli.node, req).await?;
            let hash = res["result"]["state_root_hash"]
                .as_str()
                .ok_or("state_root_hash not found in response")?;
            println!("{}", hash);
        }
        Commands::GetDeploy { hash } => {
            // Check transaction_hash (v2) with Version1/Deploy wrapper, and fallback to legacy deploy_hash dynamically
            let mut req = RpcRequest::new("info_get_transaction", json!({
                "transaction_hash": { "Version1": hash }
            }));
            let mut res = post_rpc(&client, &cli.node, req).await;

            if res.is_err() {
                // Try Deploy wrapper
                req = RpcRequest::new("info_get_transaction", json!({
                    "transaction_hash": { "Deploy": hash }
                }));
                res = post_rpc(&client, &cli.node, req).await;
            }

            if res.is_err() {
                // Try legacy info_get_deploy (legacy nodes)
                req = RpcRequest::new("info_get_deploy", json!({ "deploy_hash": hash }));
                res = post_rpc(&client, &cli.node, req).await;
            }

            let val = res?;
            println!("{}", serde_json::to_string_pretty(&val["result"])?);
        }
        Commands::GetAccountPurse { account } => {
            let clean_hash = account.trim_start_matches("account-hash-");
            let state_root_req = RpcRequest::new("chain_get_state_root_hash", json!([]));
            let state_root_res = post_rpc(&client, &cli.node, state_root_req).await?;
            let state_root_hash = state_root_res["result"]["state_root_hash"]
                .as_str()
                .ok_or("state_root_hash not found")?;

            let formatted_account = format!("account-hash-{}", clean_hash);
            let state_req = RpcRequest::new("state_get_item", json!({
                "state_root_hash": state_root_hash,
                "key": formatted_account,
                "path": []
            }));
            let state_res = post_rpc(&client, &cli.node, state_req).await?;
            let uref = state_res["result"]["stored_value"]["Account"]["main_purse"]
                .as_str()
                .ok_or("main_purse not found in account state")?;
            println!("{}", uref);
        }
        Commands::GetBalance { uref } => {
            let state_root_req = RpcRequest::new("chain_get_state_root_hash", json!([]));
            let state_root_res = post_rpc(&client, &cli.node, state_root_req).await?;
            let state_root_hash = state_root_res["result"]["state_root_hash"]
                .as_str()
                .ok_or("state_root_hash not found")?;

            let req = RpcRequest::new("state_get_balance", json!({
                "state_root_hash": state_root_hash,
                "purse_uref": uref
            }));
            let res = post_rpc(&client, &cli.node, req).await?;
            let balance = res["result"]["balance_value"]
                .as_str()
                .ok_or("balance_value not found")?;
            println!("{}", balance);
        }
        Commands::QueryDictionary { contract, dict_name, key } => {
            let state_root_req = RpcRequest::new("chain_get_state_root_hash", json!([]));
            let state_root_res = post_rpc(&client, &cli.node, state_root_req).await?;
            let state_root_hash = state_root_res["result"]["state_root_hash"]
                .as_str()
                .ok_or("state_root_hash not found")?;

            let clean_contract = contract.trim_start_matches("hash-").trim_start_matches("contract-");
            let contract_key = format!("hash-{}", clean_contract);

            let req = RpcRequest::new("state_get_dictionary_item", json!({
                "state_root_hash": state_root_hash,
                "dictionary_identifier": {
                    "ContractNamedKey": {
                        "key": contract_key,
                        "dictionary_name": dict_name,
                        "dictionary_item_key": key
                    }
                }
            }));
            let res = post_rpc(&client, &cli.node, req).await?;
            println!("{}", serde_json::to_string_pretty(&res["result"])?);
        }
        Commands::DeployWasm { wasm, secret_key, algo, chain, payment } => {
            // Find the helper binary
            let current_dir = std::env::current_dir()?;
            let helper_path = current_dir.join("swarm/casper-client/go-signer/casper-tx-signer");
            let helper_path_str = if helper_path.exists() {
                helper_path.to_string_lossy().into_owned()
            } else {
                "./casper-tx-signer".to_string()
            };

            let output = Command::new(&helper_path_str)
                .arg("--mode").arg("deploy-wasm")
                .arg("--node").arg(&cli.node)
                .arg("--chain").arg(&chain)
                .arg("--secret-key").arg(&secret_key)
                .arg("--algo").arg(&algo)
                .arg("--payment").arg(&payment)
                .arg("--wasm").arg(&wasm)
                .output()?;

            if !output.status.success() {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Deploy transaction submission failed: {}", err_msg).into());
            }

            let tx_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{}", tx_hash);
        }
        Commands::CallContract { contract, entrypoint, args, secret_key, algo, chain, payment } => {
            let current_dir = std::env::current_dir()?;
            let helper_path = current_dir.join("swarm/casper-client/go-signer/casper-tx-signer");
            let helper_path_str = if helper_path.exists() {
                helper_path.to_string_lossy().into_owned()
            } else {
                "./casper-tx-signer".to_string()
            };

            let mut cmd = Command::new(&helper_path_str);
            cmd.arg("--mode").arg("call-entrypoint")
                .arg("--node").arg(&cli.node)
                .arg("--chain").arg(&chain)
                .arg("--secret-key").arg(&secret_key)
                .arg("--algo").arg(&algo)
                .arg("--payment").arg(&payment)
                .arg("--contract-hash").arg(&contract)
                .arg("--entrypoint").arg(&entrypoint);

            if !args.is_empty() {
                cmd.arg("--args").arg(&args);
            }

            let output = cmd.output()?;

            if !output.status.success() {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Contract call execution failed: {}", err_msg).into());
            }

            let tx_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{}", tx_hash);
        }
    }

    Ok(())
}
