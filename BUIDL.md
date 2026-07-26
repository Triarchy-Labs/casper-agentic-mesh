```
◢◤￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣◥◣

            the triarchy agentic mesh
         casper agentic buildathon 2026

◥◣＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿◢◤
```

## Agents don't trust each other — the chain enforces the verdict.

### ◢◤ The Triarchy Vision: Orchestrating the Agent Economy

The Casper hackathon presented clear vectors: RWA Oracles, Yield Routers, and DAOs. But AI wasn't meant to fragment the user experience — it was meant to make complex systems seamless. We didn't just build a single bot; we built an **Agentic Mesh**.

**The Real-World Problem:** The freelance and agent economy runs on blind trust. Triarchy replaces "trust me" with an autonomous machine-to-machine bounty economy. A personal AI assistant bridges the web interface directly to the on-chain backend — you don't juggle credentials or jump between tabs. You define a goal, pay the **x402 micropayment**, and the Mesh executes.

**Deterministic Code > Probabilistic AI:** we do not blindly trust an LLM. The LLM argues; **verifiable Rust code and a WASM contract settle.** In plain words:

- **LOCKED** — CSPR sits in the escrow contract's own purse; nobody can move it out-of-band.
- **JUDGED** — an adversarial court of 5 real LLMs (prosecutor, defender, 3 jurors, chief judge) argues every submission and votes.
- **ENFORCED** — the contract's only money paths are `release` → registered hunter or `refund` → creator. **Any** verdict, hallucinated or not, can do nothing else. Trust is written in code, not promised.

> **⏱ Judges — start here: [PLAYBOOK.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/PLAYBOOK.md)** — verify everything yourself in ~10 minutes (browser-only path: 2 minutes). No marketing, only commands and live hashes.

Verifiable, not simulated — every hash below opens on the block explorer.

---

## 🗺️ The whole mesh in one board (every box is live)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  BROWSER · the cinematic gateway (Next.js, deploys to Vercel)                  │
│   Home · 5 vector dossiers ┐   Dashboard · Mesh Control ┐   Bounties · board ┐ │
│   hero live-ledger strip   │   Casper Live Stream       │   KPIs · terminal  │ │
│   ONE CLICK hub ───────────┼── Try it live (4 probes) ──┼── AI assistant     │ │
│   WALLET (Casper Wallet) ──┘                            └────────────────────┘ │
└───────────────┬────────────────────────────────────────────────────────────────┘
                │ every panel calls a real gateway route ↓
┌───────────────┴────────────────────────────────────────────────────────────────┐
│  GATEWAY API (Next.js edge · server-side keys, never shipped to the browser)   │
│   /api/onchain ──► live ledger: oracle price, agent reputation, RWA peg        │
│   /api/hire ─────► x402 gate: 402 unless a real CSPR tx is verified on-ledger  │
│   /api/casper-stream ─► CSPR.cloud feed: real blocks + deploys                 │
│   /api/mcp ──────► Model Context Protocol manifest — agent tool discovery      │
│   /api/agents ·  /api/telemetry ·  /api/tower ·  /api/tribunal                 │
└──────┬───────────────────────────────┬───────────────────────────┬─────────────┘
       │ signs & verifies              │ deliberates               │ reads / writes
┌──────┴───────────┐        ┌──────────┴──────────┐      ┌─────────┴──────────────┐
│  x402 PAYMENT    │        │  AI AGENT SWARM     │      │  CASPER TESTNET (WASM) │
│  wallet ─sign─►  │        │  compiled Rust      │      │  escrow contract       │
│  verify on-chain │        │  bounty-judge       │      │  deposit-proxy         │
│  → /api/hire     │        │  tribunal (5 LLMs)  │      │  RWA oracle contract   │
│                  │        │  tower · rwa-oracle │      │                        │
└──────────────────┘        └─────────┬───────────┘      └───────────┬────────────┘
                                      │ verdict → release / refund   │
                                      └── moves CSPR on-chain ────────┘
                                          the contract's ONLY two exits

╔════════════════════════════════════════════════════════════════════════════════╗
║  CASPER AI TOOLKIT — official, integrated with attribution (not reinvented)     ║
║   x402 (make-software) · MCP (msanlisavas) · EIP-712 (casper-ecosystem)         ║
║   Odra 0.8 · CSPR.cloud   ── our Boost Layer wraps these: hackathon_boost_layer/ ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

**The product loop, end to end:** `WALLET connect → describe a task → sign a real CSPR payment → /api/hire verifies it on the ledger → the 5-LLM court rules → the contract pays the hunter or refunds you.` No step is mocked; the contract has exactly two money paths and a verdict can pick nothing else.

---

／

##   Live on Casper Testnet — verifiable, not simulated

＼

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

## ― Autonomous agent loop ―

## executed live (AI decision → on-chain payout)

The `bounty-judge` agent asks an LLM (OpenRouter) to APPROVE/REJECT a submitted proof, and **on approval autonomously submits a real `release_bounty` transaction** that pays the hunter. A weak proof was REJECTED (no funds moved); a substantive proof was APPROVED and paid out:

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
| **Casper AI Toolkit** | x402 (live 402 gate + make-software facilitator), MCP (live `/api/mcp` + community server + our TS server), Odra 0.8, EIP-712, CSPR.cloud live stream — integrated with attribution |
| **Real-World / RWA** | live RWA oracle (real CSPR/USD on-chain) + RWA-pegged pricing |
| **UX & Design** | cinematic-brutalism gateway with live on-chain panels + one-touch ONE CLICK hub |
| **Innovation** | adversarial agent court + Antifragile Mesh (Proof-of-Liveness) — primitives the brief never named |
| **Long-Term Launch** | CI, LICENSE, [LAUNCH.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/LAUNCH.md), full [VISION.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/VISION.md) roadmap |

**Beyond the brief:** the buildathon's four examples covered RWA oracles and DAO governance; we also shipped two **new** primitives — an adversarial **Tribunal** and the **Antifragile Mesh** — plus **The Tower** overseer that turns the swarm into one organism. Full vector map: [VISION.md](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/VISION.md).

---

## What is actually built (and what is roadmap)

We separate shipped reality from vision on purpose — judges should be able to trust every claim.

### Shipped & on-chain

- **Escrow smart contract** — native `casper-contract` (`#![no_std]`), with `init` / `register_agent` / `create_bounty` / `release_bounty` / `refund_bounty`. Source: [`contracts/casper-mesh-contract/src/lib.rs`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/contracts/casper-mesh-contract/src/lib.rs).
- **Session deposit proxy** — the canonical Casper escrow-funding pattern (a stored contract cannot withdraw from a caller's main purse), at [`contracts/deposit-proxy/src/lib.rs`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/contracts/deposit-proxy/src/lib.rs).
- **Signer / RPC client** — a Go-backed `TransactionV1` signer (`swarm/casper-client/go-signer`) and a Rust JSON-RPC client (`swarm/casper-client`): deploy-wasm, call-entrypoint, session-wasm, balance/dictionary queries.
- **Gateway (Next.js)** — [`gateway`](https://github.com/Triarchy-Labs/casper-agentic-mesh/tree/main/gateway): a dashboard of live on-chain panels, **Casper Wallet connection** (modern `CasperWalletProvider`), and **server-side payment verification with no bypass** — a payment is valid only if its transaction is found on the ledger and executed successfully. Browser payments are real transfers signed by Casper Wallet via `casper-js-sdk`.
- **Bounty Judge agent** — the core agentic loop ([`swarm/bounty-judge`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/swarm/bounty-judge/src/main.rs)): it asks an LLM (via OpenRouter) to APPROVE/REJECT a submitted proof, and on approval autonomously submits a real `release_bounty` transaction that pays the hunter. The verdict is a real model call and the payout is a real signed tx — with no key or a failed call it aborts, never inventing a verdict or a hash.

  ```bash
  OPENROUTER_API_KEY=sk-... cargo run -p bounty-judge -- \
    --task-id bounty-alpha-004 \
    --description "Optimize the AST hypergraph for the Odra escrow modules." \
    --proof https://github.com/Triarchy-Labs/casper-agentic-mesh/pull/1
  ```

- **RWA Oracle contract + agent** — on-chain data feed, agent identity, reputation and an event log ([`contracts/oracle-contract`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/contracts/oracle-contract/src/lib.rs)); the `rwa-oracle` agent posts a **real CSPR/USD price** on-chain. Drives RWA-pegged bounty pricing in the UI. Package [`16d86943…64b2`](https://testnet.cspr.live/contract-package/16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2) · feed tx [`da7ac22b…9300`](https://testnet.cspr.live/transaction/da7ac22bc69c801a3600d43d408a29c85170f9205d224c3345b3f482d1949300).
- ◤ **Triarchy Tribunal** ◢ an adversarial court of real models (prosecutor, defender, a jury of diverse LLMs, a chief judge) that rules on a bounty and moves CSPR on-chain (`release`/`refund`), anchoring each verdict on the oracle ([`swarm/tribunal`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/swarm/tribunal/src/main.rs)). Both paths exercised on testnet: REJECT→refund [`4664e97a…4d03`](https://testnet.cspr.live/transaction/4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03) · APPROVE→release [`70213268…c375`](https://testnet.cspr.live/transaction/702132683a246c1e07e7c49f0e403b680d85b7114b8ec25772af5991a959c375). Fault-tolerant: partial bench → "indicative, not fully precise"; all agents down → "functions frozen, no funds moved"; `--dry-run` deliberates without spending.
- ◤ **The Tower** ◢ an overseer meta-agent that reads the whole on-chain world and dispatches sub-agents ([`swarm/tower`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/swarm/tower/src/main.rs)). Read-only, click-triggered, never an autonomous background loop.
- ◤ **Antifragile Mesh (Proof-of-Liveness)** ◢ agents post an on-chain heartbeat; if one goes dark the Tower nominates a reputation-ranked successor and the Tribunal ratifies — open escrows are rescued, never frozen. Original primitive, live on-chain: heartbeat [`b8a051a6…c89a`](https://testnet.cspr.live/transaction/b8a051a6626e1a3b82e610eb0ab4464e58ae7e3c3bee6ecf16b219eff7f4c89a).
- ◤ **Mesh Control UI** ◢ the dashboard surfaces all of the above as click-triggered panels in the Casper aesthetic ([`gateway/src/components/MeshControl.tsx`](https://github.com/Triarchy-Labs/casper-agentic-mesh/blob/main/gateway/src/components/MeshControl.tsx)), backed by `/api/tower`, `/api/tribunal` (dry-run) and `/api/onchain` (live reads). The **Casper Live Stream** panel scrolls real testnet blocks + deploys.

### Casper AI Toolkit — integrated, not reinvented

The Buildathon ships an official [AI Toolkit](https://www.casper.network/ai) and asks builders to use it. We do — with clear attribution — and add our own edge layer on top ([`hackathon_boost_layer/`](https://github.com/Triarchy-Labs/casper-agentic-mesh/tree/main/hackathon_boost_layer)). **We claim authorship only of the mesh and the Boost Layer; the upstream toolkit repos are the ecosystem's work, credited here — not re-committed as ours.**

- **x402 micropayments — LIVE.** The gateway enforces HTTP 402 pay-per-call with on-chain verification (try it: `POST /api/hire` with no header → a real `402`). We integrate the official facilitator ([make-software/casper-x402](https://github.com/make-software/casper-x402)) and ship a lightweight Next.js edge middleware in the Boost Layer — the terminal UI intercepts the request and prints `[SYS] 402 PAYMENT REQUIRED` before the AI ever sees the prompt.
- **MCP servers — LIVE.** The gateway exposes a Model Context Protocol discovery manifest at [`/api/mcp`](https://casper-agentic-mesh.vercel.app/api/mcp) — the ONE CLICK assistant fetches it live and shows the real tools. We integrate the community server ([msanlisavas/casper-mcp](https://github.com/msanlisavas/casper-mcp), 98 tools — our `StringBuilder` / `CancellationToken` optimization upstreamed) and ship our own TypeScript `@modelcontextprotocol/sdk` server (`verify_l402_payment`, `get_account_balance`) with a 3-node failover RPC pool.
- **CSPR.cloud — LIVE.** The dashboard's **Casper Live Stream** renders real testnet blocks + deploys through the official CSPR.cloud API via server-side `/api/casper-stream` (the key stays server-side; if the feed drops it says so, never faking data).
- **Odra 0.8 — used.** A batched-write oracle contract (50 assets / tx) alongside the native `#![no_std]` one.
- **EIP-712 — used.** Typed-data signatures via [casper-ecosystem/casper-eip-712](https://github.com/casper-ecosystem/casper-eip-712), cross-verified between the Go signer and the backend.

### Roadmap (clearly not yet on-chain)

**Near-term — Casper-native primitives the toolkit already endorses:**

- **Delegated agent custody** — native associated keys + weighted thresholds: a human owner grants an agent a spending key and can revoke it anytime. Trust-minimized custody of an AI agent.
- **Autonomous succession, executed** — turn the Tower's dry-run Proof-of-Liveness into real on-chain reassignment: a dead agent's open escrows pass to the highest-reputation live successor, ratified by the Tribunal.
- **The Operator** — the ONE CLICK assistant graduating from a live tool-tracer into a full mesh operator: dispatch a task to the swarm, pull any verdict's transcript, a first-class kill-switch — every action behind explicit human confirmation.
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

**Deploying the gateway (Vercel):** import this repo, set **Root Directory = `gateway`**, and add the env vars (`CASPER_RPC_URL`, `CSPR_CLOUD_API_KEY` for the live stream). `/api/onchain`, `/api/hire`, `/api/mcp` and `/api/casper-stream` work on serverless as-is. The `/api/tower` and `/api/tribunal` routes spawn the compiled Rust agents, so they need those binaries present — run the gateway on a host/VM for the live Tower/Tribunal buttons; on pure serverless they degrade gracefully ("functions frozen — we are working on it").

---

*Built for the Casper Agentic Buildathon 2026. On-chain claims are verifiable; roadmap items are labeled as such.*

`// triarchy labs`
