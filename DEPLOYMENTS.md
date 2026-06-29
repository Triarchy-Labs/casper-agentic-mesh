# On-Chain Deployment Proof — Casper Testnet

> Every entry below is a **real, executed transaction on Casper Testnet** (`casper-test`,
> network API 2.0.0). All have `error: None`. Nothing here is mocked or simulated.
> Verify any of them yourself on the block explorer.

## Contract

| Item | Value |
|------|-------|
| Contract source | [`contracts/casper-mesh-contract/src/lib.rs`](contracts/casper-mesh-contract/src/lib.rs) |
| Deployed WASM | `contracts/casper-mesh-contract/escrow_casper_ready.wasm` (bulk-memory lowered for Casper VM) |
| **Contract package hash** | `a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d` |
| Contract entity hash | `2609820f1a4c445a7cf90bc103cc7cc3dc13b0c0b42c0c94f069f986fda9414f` |
| Deployer account | `account-hash-334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867` |
| Deployer public key | `013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35` |

Explorer:
- Account: https://testnet.cspr.live/account/013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35
- Contract package: https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d

## Transactions (full escrow lifecycle, all `error: None`)

| # | Action | Transaction hash | Explorer |
|---|--------|------------------|----------|
| 1 | Deploy contract WASM (install) | `df8515855c98612e793ec30857ba9bd5cc27354f188e6e35608722df8ffe9815` | https://testnet.cspr.live/transaction/df8515855c98612e793ec30857ba9bd5cc27354f188e6e35608722df8ffe9815 |
| 2 | `init` — create dicts + escrow purse | `ac7602d1f2f5518f4b45d1828588682a21562f40a8b6baafee44928efc8ef162` | https://testnet.cspr.live/transaction/ac7602d1f2f5518f4b45d1828588682a21562f40a8b6baafee44928efc8ef162 |
| 3 | `register_agent` — on-chain agent registry write | `27a7009489008d32d6fe463540ec5322423fcd4f1c0413f9eb67f27342e0a150` | https://testnet.cspr.live/transaction/27a7009489008d32d6fe463540ec5322423fcd4f1c0413f9eb67f27342e0a150 |
| 4 | `create_bounty` via session deposit proxy — **lock 10 CSPR into escrow** | `4ad2744e9beeb6b6ae161948a03cc97f34dd58744c87e05e64836227d1d4492a` | https://testnet.cspr.live/transaction/4ad2744e9beeb6b6ae161948a03cc97f34dd58744c87e05e64836227d1d4492a |
| 5 | `release_bounty` — **pay 10 CSPR out of escrow to hunter** | `1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14` | https://testnet.cspr.live/transaction/1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14 |
| 6 | `refund_bounty` — **return locked CSPR to creator** | `895eb5531398c44a85554c11c622d3f528ef73ac9e541619f163ec392e120d87` | https://testnet.cspr.live/transaction/895eb5531398c44a85554c11c622d3f528ef73ac9e541619f163ec392e120d87 |

## State machine — verified by on-chain read-back

Bounties read directly from the contract's `bounties_dict` after execution
(`state_get_dictionary_item`), proving the full state machine, not just tx success:

| Bounty | Amount | On-chain status |
|--------|--------|-----------------|
| `bounty-alpha-003` | 10 CSPR | `1` = Released/Completed |
| `bounty-alpha-002` | 10 CSPR | `2` = Refunded |

Both terminal paths (`Locked → Released` and `Locked → Refunded`) are demonstrated
on testnet. Read-back uses dict uref `uref-112d975c…1848b75c-007`.

## Autonomous agent loop — executed live (AI decision → on-chain payout)

The `bounty-judge` agent ([`swarm/bounty-judge`](swarm/bounty-judge/src/main.rs))
ran end-to-end against a freshly locked bounty (`bounty-alpha-004`, 10 CSPR):

1. A weak proof (bare PR link) → the LLM **REJECTED** it, no funds moved.
2. A substantive proof (metrics, tests, reproducible benchmark) → the LLM
   **APPROVED** it, and the agent autonomously submitted `release_bounty`.

Real model call (OpenRouter, `anthropic/claude-opus-4.8-fast`); real on-chain payout:

| Step | Transaction | Explorer |
|------|-------------|----------|
| Lock bounty-alpha-004 (session deposit) | `afa56c8b780a7d1db35a1e47bb505d1e37439d61ba46a453c339e30e56aa04e5` | https://testnet.cspr.live/transaction/afa56c8b780a7d1db35a1e47bb505d1e37439d61ba46a453c339e30e56aa04e5 |
| **Agent-driven `release_bounty`** (after LLM APPROVE) | `c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89` | https://testnet.cspr.live/transaction/c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89 |

```bash
OPENROUTER_API_KEY=sk-... cargo run -p bounty-judge -- \
  --task-id bounty-alpha-004 --description "<task>" --proof "<submitted proof>"
```

## How to reproduce

```bash
# 1. Build the contract (native casper-contract, no_std)
cd contracts/casper-mesh-contract
cargo build --release --target wasm32-unknown-unknown

# 2. Lower bulk-memory / sign-ext so the Casper VM accepts the module
#    (Rust >= 1.82 emits memory.copy even with -C target-feature=-bulk-memory)
wasm-opt target/wasm32-unknown-unknown/release/casper_agentic_mesh_contract.wasm \
  --llvm-memory-copy-fill-lowering --signext-lowering -O2 \
  --disable-bulk-memory --disable-sign-ext -o escrow_casper_ready.wasm

# 3. Deploy (TransactionV1, Limited pricing mode — Casper 2.0 / Condor)
cd ../..
./swarm/casper-client/go-signer/casper-tx-signer --mode deploy-wasm \
  --node https://node.testnet.casper.network/rpc --chain casper-test \
  --secret-key swarm/casper-client/key.pem --payment 250000000000 \
  --wasm contracts/casper-mesh-contract/escrow_casper_ready.wasm

# 4. init, then the escrow lifecycle — see run_demo.sh for the full sequence.
```

## Notes on real Casper engineering (not theater)

- **Bulk-memory:** the Casper VM (wasmparser) rejects the bulk-memory proposal.
  Modern Rust emits `memory.copy`/`memory.fill` regardless of target-feature flags,
  so we lower them to MVP loops with `wasm-opt --llvm-memory-copy-fill-lowering`.
  The old `patch_wasm.py` replaced these opcodes with `unreachable`, which traps at
  runtime as soon as a string is copied — that approach is removed.
- **Escrow funding requires session code:** a stored contract cannot withdraw from a
  caller's main purse (Casper denies main-purse access in contract context). Funding
  therefore runs through [`contracts/deposit-proxy`](contracts/deposit-proxy/src/lib.rs):
  session code creates a temp purse, funds it from the main purse, and hands that purse
  to `create_bounty`. This is the canonical Casper escrow pattern.
- **Identity format:** `runtime::get_caller().to_string()` yields the bare account-hash
  hex (no `account-hash-` prefix); `verifier`/`hunter` args must match that form.

---

# RWA Oracle + Tribunal + The Tower — additional live proofs

## Oracle contract (RWA data + on-chain identity + reputation + events)
| Item | Value |
|------|-------|
| Source | [`contracts/oracle-contract/src/lib.rs`](contracts/oracle-contract/src/lib.rs) |
| Package hash | `16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2` |
| Explorer | https://testnet.cspr.live/contract-package/16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2 |

| Action | Tx | error |
|--------|----|-------|
| deploy oracle | `9cb3aec7163191d040ce2dcc5943d92252278be1ab6c20f13408cf0fd9e31801` | None |
| init | `ef39ea85e464e4927fabd137607f6625d69a849837bc046e8781c8820ecda071` | None |
| register_oracle | `e0c9db09b283c3ac2251d3e7d72a869b860d525180a9c1255ec08034edcce861` | None |
| post_reading (manual) | `61e2733950ee701e203cbaecd02077ac08a6a26f4544344640b283dc63248c9b` | None |
| **RWA agent: real CSPR/USD feed → on-chain** | `da7ac22bc69c801a3600d43d408a29c85170f9205d224c3345b3f482d1949300` | None |

Read-back confirmed: `READING[CSPR-USD]=value=1825…`, `REPUTATION` accrues per post.

## Triarchy Tribunal — adversarial court (real LLMs → on-chain settlement)
Prosecutor + Defender + 3 jurors (GPT-4o-mini, Llama-3.3-70B, Mistral) + Chief Judge
(Claude Opus). Both verdict paths exercised on-chain:

| Verdict | Tx | error |
|---------|----|-------|
| REJECT → refund (bounty-006) | `4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03` | None |
| verdict anchored (oracle) | `9d94e6878b0c8215cff1ee52b8cc639963246da5f2aa3b869cc5bd5b9a101ed3` | None |
| **APPROVE → release (bounty-009)** | `702132683a246c1e07e7c49f0e403b680d85b7114b8ec25772af5991a959c375` | None |
| verdict anchored (oracle) | `723b4a5c3cbbf3f7df81db11e7280b11f6012940cf4b61145de1a97cb23d6b13` | None |

Fault-tolerant: partial bench → "indicative, not fully precise"; all agents down →
"functions frozen, no funds moved"; `--dry-run` deliberates without spending.

## The Tower + Antifragile Mesh (Proof-of-Liveness) — external primitive
The overseer ([`swarm/tower`](swarm/tower/src/main.rs)) scans the on-chain world and
applies Proof-of-Liveness. Live heartbeat anchored on-chain:

| Action | Tx | error |
|--------|----|-------|
| agent heartbeat (post_reading HEARTBEAT-<agent>) | `b8a051a6626e1a3b82e610eb0ab4464e58ae7e3c3bee6ecf16b219eff7f4c89a` | None |

Before heartbeat → Tower flags "NO HEARTBEAT → autonomous succession"; after →
"ALIVE → mesh healthy". Read-only, click-triggered, no funds moved.
