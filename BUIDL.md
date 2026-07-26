```
◢◤￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣◥◣

            the triarchy agentic mesh
         casper agentic buildathon 2026

◥◣＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿◢◤
```

## Agents don't trust each other — the chain enforces the verdict.

Triarchy Agentic Mesh is an autonomous machine-to-machine bounty economy on the **Casper Network**: agents register on-chain, escrow CSPR for tasks, an adversarial AI tribunal rules on the work, and the contract settles. In plain words:

- **LOCKED** — CSPR sits in the escrow contract's purse; nobody can move it out-of-band.
- **JUDGED** — an adversarial court of 5 real LLMs (prosecutor, defender, 3 jurors, chief judge) argues every submission and votes.
- **ENFORCED** — the contract's only money paths are `release` → registered hunter or `refund` → creator. **Any** verdict, hallucinated or not, can do nothing else. Trust is written in code, not promised.

> **⏱ Judges — start here: [PLAYBOOK.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/PLAYBOOK.md)** — verify everything yourself in ~10 minutes (browser-only path: 2 minutes). No marketing, only commands and live hashes.

Verifiable, not simulated — every hash below opens on the block explorer.

---

## ⛓️ Live on Casper Testnet — verifiable, not simulated

The escrow contract is deployed and the **full bounty lifecycle has executed on-chain**. Every hash below opens on the block explorer. Full ledger, reproduction steps and engineering notes: **[DEPLOYMENTS.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/DEPLOYMENTS.md)**.

| Action | Result | Explorer |
|--------|--------|----------|
| Deploy escrow contract | `error: None` | [`df851585…e9815`](https://testnet.cspr.live/transaction/df8515855c98612e793ec30857ba9bd5cc27354f188e6e35608722df8ffe9815) |
| `init` (dicts + escrow purse) | `error: None` | [`ac7602d1…f162`](https://testnet.cspr.live/transaction/ac7602d1f2f5518f4b45d1828588682a21562f40a8b6baafee44928efc8ef162) |
| `register_agent` | `error: None` | [`27a70094…a150`](https://testnet.cspr.live/transaction/27a7009489008d32d6fe463540ec5322423fcd4f1c0413f9eb67f27342e0a150) |
| `create_bounty` — **lock 10 CSPR in escrow** | `error: None` | [`4ad2744e…492a`](https://testnet.cspr.live/transaction/4ad2744e9beeb6b6ae161948a03cc97f34dd58744c87e05e64836227d1d4492a) |
| `release_bounty` — **pay 10 CSPR from escrow** | `error: None` | [`1ea27a03…1185`](https://testnet.cspr.live/transaction/1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14) |
| `refund_bounty` — **return locked CSPR** | `error: None` | [`895eb553…0d87`](https://testnet.cspr.live/transaction/895eb5531398c44a85554c11c622d3f528ef73ac9e541619f163ec392e120d87) |

**Contract package:** [`a7e6a383…4f6d`](https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d)

Both terminal paths (`Locked → Released` and `Locked → Refunded`) are proven by on-chain dictionary **read-back** — the full state machine, not just tx success.

```bash
./run_demo.sh   # performs a real on-chain register_agent and prints live tx links
```

---

## 🤖 Autonomous agent loop — executed live (AI decision → on-chain payout)

The `bounty-judge` agent asks an LLM (OpenRouter, `anthropic/claude-opus-4.8-fast`) to APPROVE/REJECT a submitted proof, and **on approval autonomously submits a real `release_bounty` transaction** that pays the hunter. A weak proof was REJECTED (no funds moved); a substantive proof was APPROVED and paid out:

| Step | Result | Explorer |
|------|--------|----------|
| Lock `bounty-alpha-004` (session deposit) | `error: None` | [`afa56c8b…04e5`](https://testnet.cspr.live/transaction/afa56c8b780a7d1db35a1e47bb505d1e37439d61ba46a453c339e30e56aa04e5) |
| **Agent-driven `release_bounty`** (after LLM APPROVE) | `error: None` | [`c333c9d1…cb89`](https://testnet.cspr.live/transaction/c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89) |

The verdict is a real model call and the payout is a real signed tx — with no key or a failed call it aborts, never inventing a verdict or a hash.

---

## Scorecard (self-assessed against the buildathon criteria)

| Criterion | Where it's proven |
|-----------|-------------------|
| **Working Smart Contracts** | escrow + oracle deployed; full state machine + read-back ([DEPLOYMENTS.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/DEPLOYMENTS.md)) |
| **Technical Execution** | green workspace tests, CI, contracts build, gateway typechecks |
| **Use of AI / Agentic** | bounty-judge + multi-model Tribunal + Tower overseer, all real, on-chain settlement |
| **Casper AI Toolkit** | x402 (live 402 gate + make-software facilitator), MCP (live `/api/mcp` + community server + our TS server), Odra 0.8, EIP-712, CSPR.cloud — integrated with attribution |
| **Real-World / RWA** | live RWA oracle (real CSPR/USD on-chain) + RWA-pegged pricing |
| **UX & Design** | cinematic-brutalism gateway with live on-chain panel + click-triggered Mesh Control |
| **Innovation** | adversarial agent court + Antifragile Mesh (Proof-of-Liveness) — primitives the brief never named |
| **Long-Term Launch** | CI, LICENSE, [LAUNCH.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/LAUNCH.md), full [VISION.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/VISION.md) roadmap |

**Beyond the brief:** the buildathon's four examples covered RWA oracles and DAO governance; we also shipped two **external** primitives — an adversarial **Tribunal** and the **Antifragile Mesh** — plus **The Tower** overseer that turns the swarm into one organism. Full vector map: [VISION.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/VISION.md).

---

## What is actually built (and what is roadmap)

We separate shipped reality from vision on purpose — judges should be able to trust every claim.

### Shipped & on-chain

- **Escrow smart contract** — native `casper-contract` (`#![no_std]`), with `init` / `register_agent` / `create_bounty` / `release_bounty` / `refund_bounty`. Source: [`contracts/casper-mesh-contract/src/lib.rs`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/contracts/casper-mesh-contract/src/lib.rs).
- **Session deposit proxy** — the canonical Casper escrow-funding pattern (a stored contract cannot withdraw from a caller's main purse), at [`contracts/deposit-proxy/src/lib.rs`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/contracts/deposit-proxy/src/lib.rs).
- **Signer / RPC client** — a Go-backed `TransactionV1` signer (`swarm/casper-client/go-signer`) and a Rust JSON-RPC client (`swarm/casper-client`): deploy-wasm, call-entrypoint, session-wasm, balance/dictionary queries.
- **Gateway (Next.js)** — [`gateway`](https://github.com/Triarchy-Labs/casper-agentic-mesh/tree/main/gateway): 4-tab dashboard, Casper Wallet connection, and **server-side payment verification with no bypass** — a payment is valid only if its transaction is found on the ledger and executed successfully. Browser payments are real transfers signed by Casper Wallet via `casper-js-sdk`.
- **Bounty Judge agent** — the core agentic loop ([`swarm/bounty-judge`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/swarm/bounty-judge/src/main.rs)): it asks an LLM (via OpenRouter) to APPROVE/REJECT a submitted proof, and on approval autonomously submits a real `release_bounty` transaction that pays the hunter. The verdict is a real model call and the payout is a real signed tx — with no key or a failed call it aborts, never inventing a verdict or a hash.

  ```bash
  OPENROUTER_API_KEY=sk-... cargo run -p bounty-judge -- \
    --task-id bounty-alpha-004 \
    --description "Optimize the AST hypergraph for the Odra escrow modules." \
    --proof https://github.com/Triarchy-Labs/casper-agentic-mesh/pull/1
  ```

- **RWA Oracle contract + agent** — on-chain data feed, agent identity, reputation and an event log ([`contracts/oracle-contract`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/contracts/oracle-contract/src/lib.rs)); the `rwa-oracle` agent posts a **real CSPR/USD price** on-chain. Drives RWA-pegged bounty pricing in the UI. Package [`16d86943…64b2`](https://testnet.cspr.live/contract-package/16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2) · feed tx [`da7ac22b…9300`](https://testnet.cspr.live/transaction/da7ac22bc69c801a3600d43d408a29c85170f9205d224c3345b3f482d1949300).
- **🔥 Triarchy Tribunal** — an adversarial court of real models (prosecutor, defender, a jury of diverse LLMs, a chief judge) that rules on a bounty and moves CSPR on-chain (`release`/`refund`), anchoring each verdict on the oracle ([`swarm/tribunal`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/swarm/tribunal/src/main.rs)). Both paths exercised on testnet: REJECT→refund [`4664e97a…4d03`](https://testnet.cspr.live/transaction/4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03) · APPROVE→release [`70213268…c375`](https://testnet.cspr.live/transaction/702132683a246c1e07e7c49f0e403b680d85b7114b8ec25772af5991a959c375). Fault-tolerant: partial bench → "indicative, not fully precise"; all agents down → "functions frozen, no funds moved"; `--dry-run` deliberates without spending.
- **⛩️ The Tower** — an overseer meta-agent that reads the whole on-chain world and dispatches sub-agents ([`swarm/tower`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/swarm/tower/src/main.rs)). Read-only, click-triggered, never an autonomous background loop.
- **🧬 Antifragile Mesh (Proof-of-Liveness)** — agents post an on-chain heartbeat; if one goes dark the Tower nominates a reputation-ranked successor and the Tribunal ratifies — open escrows are rescued, never frozen. Original primitive, live on-chain: heartbeat [`b8a051a6…c89a`](https://testnet.cspr.live/transaction/b8a051a6626e1a3b82e610eb0ab4464e58ae7e3c3bee6ecf16b219eff7f4c89a).
- **Mesh Control UI** — the dashboard surfaces all of the above as click-triggered panels in the Vercel-Geist / Casper aesthetic ([`gateway/src/components/MeshControl.tsx`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/gateway/src/components/MeshControl.tsx)), backed by `/api/tower`, `/api/tribunal` (dry-run) and `/api/onchain` (live reads).

### Casper AI Toolkit — integrated, not reinvented

The Buildathon ships an official [AI Toolkit](https://www.casper.network/ai) and asks builders to use it. We do — with clear attribution — and add our own edge layer on top. **We claim authorship only of the mesh and the Boost Layer; the three upstream toolkit repos are the ecosystem's work, credited below.**

- **x402 micropayments — LIVE.** Our gateway enforces HTTP 402 pay-per-call with on-chain payment verification (no bypass; try it: `POST /api/hire` with no header → a real `402`). We integrate the official Casper x402 facilitator ([make-software/casper-x402](https://github.com/make-software/casper-x402)) and ship a lightweight Next.js edge middleware wrapper in our Boost Layer.
- **MCP servers — LIVE.** The gateway exposes a Model Context Protocol discovery manifest at [`/api/mcp`](https://casper-agentic-mesh.vercel.app/api/mcp). We integrate the community Casper MCP server ([msanlisavas/casper-mcp](https://github.com/msanlisavas/casper-mcp), 16 tool categories — our [StringBuilder/CancellationToken optimization](https://github.com/msanlisavas/casper-mcp) upstreamed) and ship our own TypeScript MCP server (`@modelcontextprotocol/sdk`) with L402-validation tools in the Boost Layer.
- **Odra Framework — used.** Our Boost Layer includes an Odra 0.8 oracle contract with batched dictionary writes, alongside the native `#![no_std]` contract.
- **EIP-712 — used.** Typed-data signatures via the official [casper-ecosystem/casper-eip-712](https://github.com/casper-ecosystem/casper-eip-712), cross-verified between our Go signer and backend.
- **CSPR.cloud — LIVE.** The dashboard's **Casper Live Stream** card renders real testnet blocks and deploys pulled from the official CSPR.cloud API through our server-side `/api/casper-stream` (the key stays server-side, never shipped to the browser). Block heights climb in real time; if the feed is unreachable it says so, never faking data.
- **Casper Wallet — LIVE.** The WALLET button connects the browser extension via the modern `CasperWalletProvider` (with legacy fallback); EXECUTE_SEQ signs a real CSPR transfer that the gateway verifies on the ledger before any AI work runs.

> Boost Layer (our original edge code over the official toolkit): [`hackathon_boost_layer/`](https://github.com/Triarchy-Labs/casper-agentic-mesh/tree/main/hackathon_boost_layer). The upstream toolkit repos are referenced by their public URLs above — not re-committed as ours.

### Roadmap (clearly not yet on-chain)

**Near-term — Casper-native primitives the toolkit already endorses:**

- **Delegated agent custody** — native associated keys + weighted thresholds: a human owner grants an agent a spending key and can revoke it anytime. Trust-minimized custody of an AI agent.
- **Autonomous succession, executed** — turn the Tower's dry-run Proof-of-Liveness into real on-chain reassignment: a dead agent's open escrows pass to the highest-reputation live successor, ratified by the Tribunal.
- **Deeper MCP + x402 autonomy** — beyond today's live `/api/mcp` manifest and 402 gate: full agent-to-agent tool discovery and pay-per-call settlement across the whole toolkit surface.
- **Upgradable compliance contracts** — native contract versioning for KYC / compliance tokens an agent can revoke or update, without exposing user data on-chain.

**Mid-term — depth & network effects:**

- **RWA oracle** with real external feeds + a risk model, and permissioned posting.
- **Portable agent credit score** — reputation any dApp can read: a network effect beyond one project.
- **Stake-weighted slashing** and a decentralized jury swarm for dispute resolution.
- **ZK proofs** of execution safety · **soulbound (CEP-78)** reputation credentials · an **L402-Casper** gateway spec.

---

## Casper engineering notes (the hard parts, done right)

- **Bulk-memory:** the Casper VM rejects the bulk-memory proposal, and modern Rust emits `memory.copy` regardless of `-C target-feature=-bulk-memory`. We lower it to MVP loops with `wasm-opt --llvm-memory-copy-fill-lowering` (binaryen ≥ 124). Replacing those opcodes with `unreachable` — as the old `patch_wasm.py` did — traps at runtime the moment a string is copied; that approach was removed.
- **Pricing mode:** transactions are built as `TransactionV1` with `Limited` pricing via the Go SDK, which matches current testnet (API 2.0.0) and avoids the "invalid pricing mode" failures seen with mismatched CLI versions.
- **Main-purse rule:** escrow funding must run as session code (see deposit proxy) because a stored contract cannot spend a caller's main purse.

---

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

Full deploy + lifecycle reproduction: **[DEPLOYMENTS.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/DEPLOYMENTS.md)**.

**Deploying the gateway (Vercel):** import this repo and set **Root Directory = `gateway`**. `/api/onchain` (live ledger reads) works on serverless as-is. The `/api/tower` and `/api/tribunal` routes spawn the compiled Rust agents, so they need those binaries present — run the gateway on a host/VM (or a small backend service) for the live Tower/Tribunal buttons; on pure serverless they degrade gracefully ("functions frozen — we are working on it").

---

*Built for the Casper Agentic Buildathon 2026. On-chain claims are verifiable; roadmap items are labeled as such.*

`// triarchy labs`
