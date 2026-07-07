# JUDGE_PROOF — one page, verify in 3 minutes

Everything below is **real and on Casper testnet** (`casper-test`, API 2.0.0). Nothing here is mocked.

## 1. Click these — the full escrow lifecycle, on-chain

| What | Transaction |
|------|-------------|
| Deploy escrow contract | [df851585…](https://testnet.cspr.live/transaction/df8515855c98612e793ec30857ba9bd5cc27354f188e6e35608722df8ffe9815) |
| `create_bounty` — lock 10 CSPR | [4ad2744e…](https://testnet.cspr.live/transaction/4ad2744e9beeb6b6ae161948a03cc97f34dd58744c87e05e64836227d1d4492a) |
| `release_bounty` — pay 10 CSPR | [1ea27a03…](https://testnet.cspr.live/transaction/1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14) |
| `refund_bounty` — return CSPR | [895eb553…](https://testnet.cspr.live/transaction/895eb5531398c44a85554c11c622d3f528ef73ac9e541619f163ec392e120d87) |

**Escrow package:** [a7e6a383…](https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d) · **Oracle package:** [16d86943…](https://testnet.cspr.live/contract-package/16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2)

## 2. The agentic moment — AI verdict moves real money

`bounty-judge` asks an LLM to APPROVE/REJECT a proof, and **on APPROVE autonomously submits a real `release_bounty`**:

| Step | Transaction |
|------|-------------|
| Lock bounty-alpha-004 | [afa56c8b…](https://testnet.cspr.live/transaction/afa56c8b780a7d1db35a1e47bb505d1e37439d61ba46a453c339e30e56aa04e5) |
| **Agent-driven release after LLM APPROVE** | [c333c9d1…](https://testnet.cspr.live/transaction/c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89) |

Tribunal (adversarial court): REJECT→refund [4664e97a…](https://testnet.cspr.live/transaction/4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03) · APPROVE→release [70213268…](https://testnet.cspr.live/transaction/702132683a246c1e07e7c49f0e403b680d85b7114b8ec25772af5991a959c375)

## 3. Run these

```bash
# Full lifecycle against live testnet (real register_agent + tx links):
./run_demo.sh

# Autonomous AI → on-chain payout:
OPENROUTER_API_KEY=sk-... cargo run -p bounty-judge -- \
  --task-id bounty-alpha-004 --description "<task>" --proof "<proof>"

# Workspace tests + contract builds:
cargo test --workspace --exclude ouroboros-brain
cargo build --release --target wasm32-unknown-unknown  # escrow + oracle
```

## 4. Shipped vs roadmap (we label it, so you can trust it)

**Shipped & on-chain:** escrow (full state machine, read-back verified) · RWA oracle (real CSPR/USD feed + reputation) · bounty-judge (AI verdict → payout) · Tribunal · The Tower + Proof-of-Liveness heartbeat · gateway with **server-side payment verification** and live `/api/onchain` reads.

**Roadmap (not yet on-chain, clearly labeled):** stake-weighted slashing · decentralized jury swarm · ZK execution proofs · soulbound (CEP-78) reputation · L402-Casper spec.

## 5. Known MVP limitations (honest)

We would rather state these than have you find them:

- **Gateway payment check (v1 → now hardened):** `lib/casper.ts` verifies the payment tx **exists, executed, pays the platform account, and meets the minimum amount** — an unrelated or unsuccessful hash is rejected. (Earlier revisions only checked existence; this is fixed.) Task↔payment memo correlation is best-effort, not yet enforced.
- **Escrow authorization (hardened in v2, deployed):** v1 required `caller == verifier` (real access control) but accepted a caller-supplied payout purse. **v2 removes that**: `release_bounty`/`refund_bounty` take no purse arg and pay the stored **hunter/creator account** directly via `transfer_from_purse_to_account`. Deployed on testnet as package [`e68eae90…`](https://testnet.cspr.live/contract-package/e68eae90f71f67851a3220e1cbe77844fbf1323a2ae5176a1ee03b9106a39449) — deploy [`3a74e651…`](https://testnet.cspr.live/transaction/3a74e65184966b02ef66419846554715498834b7960bce76040bae5c667ec66c), init + register `error None`. (v1 package `a7e6a383…` remains as the original lifecycle proof.)
- **Oracle feed:** `post_reading` is an **open MVP feed** (any caller can post + accrue reputation); a permissioned variant restricts posting to registered oracles.

## Links

- **Repo:** https://github.com/Triarchy-Labs/casper-agentic-mesh
- **Live gateway:** https://casper-agentic-mesh.vercel.app
- **Full ledger + reproduction:** [DEPLOYMENTS.md](DEPLOYMENTS.md) · **Vision/roadmap:** [VISION.md](VISION.md)
