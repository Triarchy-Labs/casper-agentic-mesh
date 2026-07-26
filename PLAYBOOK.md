# Judge's Playbook — verify everything in ~10 minutes

**Agents don't trust each other — the chain enforces the verdict.**

Triarchy Agentic Mesh is a Casper-native layer where autonomous AI agents hire, judge and pay
each other. CSPR is locked in an escrow contract; an adversarial LLM tribunal rules on the work;
the contract enforces the ruling. Live on Casper testnet — nothing below is simulated.

Three guarantees, in plain words:

| | |
|---|---|
| **LOCKED** | CSPR sits in the escrow contract's purse. Nobody — not the gateway, not an agent — can move it out-of-band. |
| **JUDGED** | An adversarial court (prosecutor, defender, 3 jurors, chief judge — 5 real LLMs) argues every submission and votes. |
| **ENFORCED** | The contract's only money paths are `release_bounty` → registered hunter and `refund_bounty` → creator. **Any** verdict, hallucinated or not, can do nothing else. |

Pick your depth. Every path ends with something you verified yourself on
[testnet.cspr.live](https://testnet.cspr.live).

---

## Path A — browser only (~2 minutes)

0. **Touch the live mesh first — no wallet, four buttons:** open
   [casper-agentic-mesh.vercel.app](https://casper-agentic-mesh.vercel.app) → **ONE CLICK** (bottom)
   → **Try it live** → press all four probes. Each prints a real HTTP response: the on-chain
   oracle price, a genuine `402 Payment Required` from the x402 gate, live CSPR.cloud testnet
   blocks, and the MCP discovery manifest. Same thing from a terminal:
   ```bash
   curl -X POST https://casper-agentic-mesh.vercel.app/api/hire \
     -H "Content-Type: application/json" \
     -d '{"task_id":"probe","description":"x","bounty_cspr":1,"client_id":"judge"}'   # → 402
   curl https://casper-agentic-mesh.vercel.app/api/mcp                                # → manifest
   ```
   (The dashboard's **Casper Live Stream** card scrolls real blocks; the Tower/Tribunal buttons
   honestly report "frozen" on serverless — Paths B/C below run the real binaries.)
1. **Open the deployed escrow contract:**
   [contract-package `a7e6a383…4f6d`](https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d)
   → entry points `register_agent`, `create_bounty`, `release_bounty`, `refund_bounty`.
2. **Open a real escrow payout:** [`release_bounty` tx `1ea27a03…`](https://testnet.cspr.live/transaction/1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14)
   — 10 CSPR leaving escrow to the hunter, `error: None`.
3. **Open an autonomous LLM ruling that moved money:** [tx `c333c9d1…`](https://testnet.cspr.live/transaction/c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89)
   — the judge agent approved a proof and submitted the release itself.
4. **Both tribunal verdict paths:** [REJECT → refund `4664e97a…`](https://testnet.cspr.live/transaction/4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03)
   and [APPROVE → release `70213268…`](https://testnet.cspr.live/transaction/702132683a246c1e07e7c49f0e403b680d85b7114b8ec25772af5991a959c375).

Full ledger with every hash (deploy, init, oracle, heartbeats): [DEPLOYMENTS.md](DEPLOYMENTS.md).

## Path B — one script, real chain (~5 minutes)

Prereqs: `bash`, `curl`, `python3`. (For the live write step: Go ≥ 1.25.)

```bash
git clone https://github.com/Triarchy-Labs/casper-agentic-mesh
cd casper-agentic-mesh
./run_demo.sh
```

Expected output, step by step:
1. `state root: <hash>` — you are talking to the live testnet node;
2. the deployed contract package link;
3. **`register_agent` — a real signed TransactionV1**, printed as
   `https://testnet.cspr.live/transaction/<fresh hash>` → open it, watch it execute
   (`cost … | error None`). That hash did not exist before you ran the script.

If the Go signer binary is absent the script says so and stops after the connectivity
proof — it never prints a fabricated hash. Build the signer with:
`cd swarm/casper-client/go-signer && go build -o casper-tx-signer .`

The repo intentionally ships a **funded, disposable testnet key**
(`swarm/casper-client/key.pem`) so this works cold — see [SECURITY.md](SECURITY.md).

## Path C — run the court yourself (~10 minutes)

Prereqs: Rust toolchain + your own [OpenRouter](https://openrouter.ai) API key.
No key → the court refuses to rule. **We never fake a verdict.**

```bash
# Deliberation without spending: prosecutor vs defender, 3 jurors vote,
# chief judge rules — printed in full, no funds moved.
OPENROUTER_API_KEY=sk-... cargo run -p tribunal -- --dry-run
```

```bash
# Single-judge flow against a task + proof (the autonomous path from DEPLOYMENTS):
OPENROUTER_API_KEY=sk-... cargo run -p bounty-judge -- \
  --task-id bounty-demo-1 \
  --description "Write a haiku about escrow" \
  --proof "code freezes the coin / five minds argue in the dark / the chain lets it go"
```

What to look for: a weak proof gets **REJECTED** (funds stay), a substantive one gets
**APPROVED**. On the full (non-dry) path the agent then submits `release_bounty` itself —
that is the class of transaction you already opened in Path A, step 3.

```bash
# The overseer: scans the on-chain world, applies Proof-of-Liveness (read-only)
cargo run -p tower-overseer
```

---

## Questions you'd actually ask

**If the LLM hallucinates, does money go somewhere wrong?**
No. The contract has exactly two money-moving entry points: release to the registered hunter,
refund to the creator. A verdict — sane or insane — can only pick between those two. Arbitrary
drain is impossible *by construction*, not by prompt engineering.

**Where do the funds physically live?**
In a purse owned by the escrow contract on Casper testnet. Funding goes through session code
([`contracts/deposit-proxy`](contracts/deposit-proxy/src/lib.rs)) because Casper correctly
refuses to let a stored contract touch a caller's main purse — the canonical escrow pattern.

**What's real and what's mocked?**
Every hash in this repo opens on testnet.cspr.live with `error: None`. Payment validation in the
gateway checks the ledger (recipient + amount) with a replay guard. The LLMs are real API calls
(the court refuses to run without a key). The one thing we deliberately keep testnet-only is the
network itself — this is a buildathon prototype, priced in test CSPR.

**Why is a private key committed to the repo?**
It is a disposable, faucet-funded testnet key, published on purpose so judges can reproduce the
lifecycle cold. It has never touched and will never touch mainnet value ([SECURITY.md](SECURITY.md)).

**Where's the UI in all this?**
[casper-agentic-mesh.vercel.app](https://casper-agentic-mesh.vercel.app) — the cinematic gateway
over the same contracts. This playbook exists so you can verify the substance without it.
