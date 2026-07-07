<div align="center">

  <h1>Triarchy Agentic Mesh — Gateway</h1>
  <p><strong>The Next.js control plane for an autonomous agent economy on Casper.</strong></p>
  <p><em>Live on-chain reads, real Casper-Wallet payments, and a zero-trust payload firewall — Casper Agentic Buildathon 2026.</em></p>

  <a href="https://casper-agentic-mesh.vercel.app/">Live Demo</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#security">Security</a>

</div>

<br/>

> This is the `gateway/` package of the [Triarchy Agentic Mesh](../README.md) monorepo. On-chain proofs (contract package hashes + full lifecycle tx hashes) live in [DEPLOYMENTS.md](../DEPLOYMENTS.md).

## What this gateway is

A 4-tab dashboard and an L402-style HTTP endpoint for a machine-to-machine bounty economy settled on the **Casper Network**:

1. **Payment** — an agent (or human, via the **Casper Wallet** extension) signs a real native CSPR transfer to the platform account (`lib/pay.ts`). No mock path.
2. **Verification** — the gateway verifies that payment **server-side against the live Casper ledger** (`lib/casper.ts`): the transaction must exist, have executed successfully, pay **to the platform account**, and meet the **minimum amount**. Any random or unrelated hash is rejected.
3. **Quarantine** — the untrusted task payload is scanned before execution (replay guard → spending policy → payload firewall).
4. **Routing** — cleared tasks are dispatched (local LLM micro-bounty → enterprise compute → P2P overflow).
5. **On-chain truth** — `/api/onchain` reads escrow, oracle and heartbeat state directly from the contracts (no cache, no fabrication).

## Security {#security}

Every payload from an untrusted agent passes a layered check:

| Layer | Mechanism | Purpose |
|-------|-----------|---------|
| L1 | On-chain payment verification (`lib/casper.ts`) | Recipient + amount + execution success on the live ledger |
| L2 | ReplayGuard (`lib/replay-guard.ts`) | 5-min TTL — each txHash is single-use |
| L3 | SpendingPolicy (`lib/spending-policy.ts`) | Per-caller allow/block list + per-call/daily/global caps |
| L4 | Payload firewall (`lib/wasm_sandbox.ts`) | Token + heuristic scan for injection / shell escape |
| L5 | SSRF protection (`lib/security.ts`) | Blocks localhost, private subnets, IPv6 on external fetches |

## Quick start {#quick-start}

```bash
git clone https://github.com/Triarchy-Labs/casper-agentic-mesh.git
cd casper-agentic-mesh/gateway
cp .env.example .env.local   # CASPER_RPC_URL, platform pubkey, OPENROUTER_API_KEY
npm install
npm run dev                  # http://localhost:3000
```

### L402 flow

```bash
# No payment -> 402
curl -X POST http://localhost:3000/api/hire -H "Content-Type: application/json" \
  -d "{\"description\":\"Summarize this paper\",\"bounty_cspr\":\"2.5\"}"

# With a REAL Casper testnet tx paying the platform account -> executes
curl -X POST http://localhost:3000/api/hire -H "Content-Type: application/json" \
  -H "x-l402-txhash: <YOUR_CASPER_TESTNET_TX_HASH>" \
  -d "{\"description\":\"Summarize this paper\",\"bounty_cspr\":\"2.5\",\"client_id\":\"demo_agent\"}"
```

## Environment

| Variable | Description | Default |
|----------|-------------|---------|
| `CASPER_RPC_URL` | Casper testnet JSON-RPC node | `https://node.testnet.casper.network/rpc` |
| `NEXT_PUBLIC_CASPER_PLATFORM_PUBKEY` | Platform account public key (receives CSPR) | deployer key |
| `CASPER_PLATFORM_ACCOUNT_HASH` | Account-hash the payment must target | derived from platform key |
| `OPENROUTER_API_KEY` | LLM routing (verdicts, Tribunal, task exec) | Required for AI paths |

## Deploying (Vercel)

Import the repo, set **Root Directory = `gateway`**. `/api/onchain` works on serverless as-is. `/api/tower` and `/api/tribunal` spawn the compiled Rust agents, so on pure serverless they degrade gracefully ("functions frozen — no funds moved").

## Stack

`Next.js 16` · `React 19` · `TailwindCSS 4` · `casper-js-sdk` (wallet payments) · `framer-motion` + `GSAP` + `Lenis` (motion) · `OpenRouter` (LLM routing).

---

© 2026 Triarchy Labs. Built for the Casper Agentic Buildathon 2026.
