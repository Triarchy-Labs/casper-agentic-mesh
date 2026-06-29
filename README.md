◢◤￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣◥◣

            the triarchy agentic mesh
         casper agentic buildathon 2026

◥◣＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿◢◤

An autonomous machine-to-machine (M2M) bounty economy on the **Casper Network**:
AI agents register on-chain, escrow CSPR for tasks, and release funds to each
other through a trustless smart contract — with an x402 payment layer and a
Next.js gateway that verifies every payment against the live ledger.

────────────────────────────────────────────────────────────────

## ✅ Live on Casper Testnet — verifiable, not simulated

The escrow contract is deployed and the **full bounty lifecycle has executed
on-chain**. Every hash below opens on the block explorer. Full ledger,
reproduction steps and engineering notes: **[DEPLOYMENTS.md](DEPLOYMENTS.md)**.

| Action | Result | Explorer |
|--------|--------|----------|
| Deploy escrow contract | ✅ `error: None` | [`df851585…e9815`](https://testnet.cspr.live/transaction/df8515855c98612e793ec30857ba9bd5cc27354f188e6e35608722df8ffe9815) |
| `init` (dicts + escrow purse) | ✅ `error: None` | [`ac7602d1…f162`](https://testnet.cspr.live/transaction/ac7602d1f2f5518f4b45d1828588682a21562f40a8b6baafee44928efc8ef162) |
| `register_agent` | ✅ `error: None` | [`27a70094…a150`](https://testnet.cspr.live/transaction/27a7009489008d32d6fe463540ec5322423fcd4f1c0413f9eb67f27342e0a150) |
| `create_bounty` — **lock 10 CSPR in escrow** | ✅ `error: None` | [`4ad2744e…492a`](https://testnet.cspr.live/transaction/4ad2744e9beeb6b6ae161948a03cc97f34dd58744c87e05e64836227d1d4492a) |
| `release_bounty` — **pay 10 CSPR from escrow** | ✅ `error: None` | [`1ea27a03…1185`](https://testnet.cspr.live/transaction/1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14) |

**Contract package:** [`a7e6a383…4f6d`](https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d)

```bash
./run_demo.sh   # performs a real on-chain register_agent and prints live tx links
```

────────────────────────────────────────────────────────────────

## Scorecard (self-assessed against the buildathon criteria)

| Criterion | Where it's proven |
|-----------|-------------------|
| Working Smart Contracts | escrow + oracle deployed; full state machine + read-back ([DEPLOYMENTS.md](DEPLOYMENTS.md)) |
| Technical Execution | green workspace tests, CI, contracts build, gateway typechecks |
| Use of AI / Agentic | bounty-judge + multi-model Tribunal + Tower overseer, all real, on-chain settlement |
| Real-World / RWA | live RWA oracle (real CSPR/USD on-chain) + RWA-pegged pricing |
| UX & Design | cinematic-brutalism gateway with live on-chain panel + click-triggered Mesh Control |
| Innovation | adversarial agent court + Antifragile Mesh (Proof-of-Liveness) — primitives the brief never named |
| Long-Term Launch | CI, LICENSE, [LAUNCH.md](LAUNCH.md), full [VISION.md](VISION.md) roadmap |

Beyond the brief: the buildathon's four examples covered RWA oracles and DAO
governance; we also shipped two **external** primitives — an adversarial **Tribunal**
and the **Antifragile Mesh** — plus **The Tower** overseer that turns the swarm into
one organism. Full vector map: [VISION.md](VISION.md).

## What is actually built (and what is roadmap)

We separate shipped reality from vision on purpose — judges should be able to
trust every claim.

### Shipped & on-chain
- **Escrow smart contract** — native `casper-contract` (`#![no_std]`), with
  `init` / `register_agent` / `create_bounty` / `release_bounty` / `refund_bounty`.
  Source: [`contracts/casper-mesh-contract/src/lib.rs`](contracts/casper-mesh-contract/src/lib.rs).
- **Session deposit proxy** — the canonical Casper escrow-funding pattern (a
  stored contract cannot withdraw from a caller's main purse), at
  [`contracts/deposit-proxy/src/lib.rs`](contracts/deposit-proxy/src/lib.rs).
- **Signer / RPC client** — a Go-backed `TransactionV1` signer
  ([`swarm/casper-client/go-signer`](swarm/casper-client/go-signer)) and a Rust
  JSON-RPC client ([`swarm/casper-client`](swarm/casper-client/src/main.rs)):
  deploy-wasm, call-entrypoint, session-wasm, balance/dictionary queries.
- **Gateway** (Next.js) — [`gateway`](gateway): 4-tab dashboard, Casper Wallet
  connection, and **server-side payment verification with no bypass**
  ([`gateway/src/lib/casper.ts`](gateway/src/lib/casper.ts)): a payment is valid
  only if its transaction is found on the ledger and executed successfully.
  Browser payments are real transfers signed by Casper Wallet via `casper-js-sdk`
  ([`gateway/src/lib/pay.ts`](gateway/src/lib/pay.ts)).
- **Swarm agents** — `x402-sniper` and `swarm-engine` submit **real signed
  transactions** through the signer (no fabricated hashes); `x402-liquidator`
  derives its health factor from **live on-chain balances** via RPC.
- **Bounty Judge agent** — the core agentic loop
  ([`swarm/bounty-judge`](swarm/bounty-judge/src/main.rs)): it asks an LLM (via
  OpenRouter) to APPROVE/REJECT a submitted proof, and **on approval autonomously
  submits a real `release_bounty` transaction** that pays the hunter. The verdict
  is a real model call and the payout is a real signed tx — with no key or a
  failed call it aborts, never inventing a verdict or a hash.

  ```bash
  OPENROUTER_API_KEY=sk-... cargo run -p bounty-judge -- \
    --task-id bounty-alpha-004 \
    --description "Optimize the AST hypergraph for the Odra escrow modules." \
    --proof https://github.com/Triarchy-Labs/casper-agentic-mesh/pull/1
  ```
- **RWA Oracle contract + agent** — on-chain data feed, agent identity,
  reputation and an event log ([`contracts/oracle-contract`](contracts/oracle-contract/src/lib.rs));
  the [`rwa-oracle`](swarm/rwa-oracle/src/main.rs) agent posts a **real CSPR/USD
  price** on-chain. Drives RWA-pegged bounty pricing in the UI.
- **🔥 Triarchy Tribunal** — an adversarial court of real models (prosecutor,
  defender, a jury of diverse LLMs, a chief judge) that rules on a bounty and
  moves CSPR on-chain (`release`/`refund`), anchoring each verdict on the oracle
  ([`swarm/tribunal`](swarm/tribunal/src/main.rs)). Fault-tolerant: partial bench →
  "indicative, not fully precise"; all agents down → "functions frozen, no funds
  moved"; `--dry-run` deliberates without spending.
- **🗼 The Tower** — an overseer meta-agent that reads the whole on-chain world and
  dispatches sub-agents ([`swarm/tower`](swarm/tower/src/main.rs)). Read-only,
  click-triggered, never an autonomous background loop.
- **🧬 Antifragile Mesh (Proof-of-Liveness)** — agents post an on-chain heartbeat;
  if one goes dark the Tower nominates a reputation-ranked successor and the
  Tribunal ratifies — open escrows are rescued, never frozen. Original primitive,
  live on-chain.
- **Mesh Control UI** — the dashboard surfaces all of the above as click-triggered
  panels in the Vercel-Geist / Casper aesthetic ([`gateway/src/components/MeshControl.tsx`](gateway/src/components/MeshControl.tsx)),
  backed by `/api/tower`, `/api/tribunal` (dry-run) and `/api/onchain` (live reads).

### Roadmap (clearly not yet on-chain)
Stake-weighted slashing, a decentralized jury swarm, ZK proofs of execution
safety, flash-loan-funded snipers, soulbound (CEP-78) reputation, and an
L402-Casper gateway spec. See the bottom of this file. The `swarm-engine`
trading/decision core is ported intelligence and uses market-data feeds (live
DexScreener, or `MOCK_DATA=1` for offline runs) — its **Casper writes are real**.

────────────────────────────────────────────────────────────────

## Casper engineering notes (the hard parts, done right)

- **Bulk-memory:** the Casper VM rejects the bulk-memory proposal, and modern
  Rust emits `memory.copy` regardless of `-C target-feature=-bulk-memory`. We
  lower it to MVP loops with `wasm-opt --llvm-memory-copy-fill-lowering`
  (binaryen ≥ 124). Replacing those opcodes with `unreachable` — as the old
  `patch_wasm.py` did — traps at runtime the moment a string is copied; that
  approach was removed.
- **Pricing mode:** transactions are built as `TransactionV1` with `Limited`
  pricing via the Go SDK, which matches current testnet (API 2.0.0) and avoids
  the "invalid pricing mode" failures seen with mismatched CLI versions.
- **Main-purse rule:** escrow funding must run as session code (see deposit
  proxy) because a stored contract cannot spend a caller's main purse.

────────────────────────────────────────────────────────────────

## Quick start

```bash
# 1. Contract → wasm → Casper-ready (lower bulk-memory)
cd contracts/casper-mesh-contract
cargo build --release --target wasm32-unknown-unknown
wasm-opt target/wasm32-unknown-unknown/release/casper_agentic_mesh_contract.wasm \
  --llvm-memory-copy-fill-lowering --signext-lowering -O2 \
  --disable-bulk-memory --disable-sign-ext -o escrow_casper_ready.wasm

# 2. Signer
cd ../../swarm/casper-client/go-signer && go build -o casper-tx-signer main.go

# 3. Live demo (real on-chain tx)
cd ../../.. && ./run_demo.sh

# 4. Gateway
cd gateway && npm install && npm run dev
```

Full deploy + lifecycle reproduction: **[DEPLOYMENTS.md](DEPLOYMENTS.md)**.

### Deploying the gateway (Vercel)
Import this repo and set **Root Directory = `gateway`**. `/api/onchain` (live ledger
reads) works on serverless as-is. The `/api/tower` and `/api/tribunal` routes spawn
the compiled Rust agents, so they need those binaries present — run the gateway on a
host/VM (or a small backend service) for the live Tower/Tribunal buttons; on pure
serverless they degrade gracefully ("functions frozen — we are working on it").

────────────────────────────────────────────────────────────────

## Scale expansion roadmap

◆ **Vector Alpha — Autonomous escrow:** stake-weighted slashable collateral;
  decentralized jury swarm for dispute resolution; streaming micropayments.
◆ **Vector Beta — Pre-trade risk oracle:** ZK proofs of execution safety;
  sentinel threat gossip; shadow-state simulation against live forks.
◆ **Vector Gamma — Cognitive arbitrage:** flash-escrow-funded snipers; agent
  guilds & sub-escrows; tokenized gas-hedging futures.
◆ **Vector Delta — Absolute synergy:** self-healing P2P load balancing;
  soulbound CEP-78 reputation credentials; an L402-Casper gateway RFC.

*Built for the Casper Agentic Buildathon 2026. On-chain claims are verifiable;
roadmap items are labeled as such.*

// triarchy labs
