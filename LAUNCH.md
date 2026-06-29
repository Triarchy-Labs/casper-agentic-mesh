# Triarchy Agentic Mesh — Launch & Go-to-Market

## What ships today (live on Casper testnet)
- Escrow contract with full bounty state machine (deploy/init/register/create/release/refund).
- RWA oracle contract: on-chain data feed, agent identity, reputation, event log.
- Autonomous agents: RWA oracle, bounty judge, and the **Triarchy Tribunal**
  (adversarial multi-model court) — all submitting real signed transactions.
- Gateway dApp with Casper Wallet payments verified against the live ledger and a
  live on-chain state panel (oracle price, agent reputation, RWA-pegged pricing).

All transactions are verifiable in [DEPLOYMENTS.md](DEPLOYMENTS.md).

## Positioning
The trust layer for the agent economy: agents that **escrow, verify, and pay each
other** with on-chain reputation and a verifiable dispute court — on Casper.

## Target users
1. **AI builders** wiring autonomous agents that must transact safely (M2M).
2. **DAOs / bounty platforms** needing trust-minimized work verification + payout.
3. **RWA / DeFi teams** needing verifiable on-chain data feeds with agent reputation.

## Go-to-market (first 90 days)
- **Week 1–2:** open-source release, demo video, Casper Discord/Telegram + DoraHacks BUIDL page, CSPR.fans community vote.
- **Week 3–6:** developer guide + `cargo`/`npm` quickstart; onboard 3 pilot bounty issuers.
- **Week 7–12:** mainnet readiness review; apply to Casper grants/incubation; publish the L402-Casper gateway spec draft.

## Socials & channels (to be filled at submission)
- GitHub: https://github.com/Triarchy-Labs/casper-agentic-mesh
- X / Twitter: @TriarchyLabs (placeholder)
- Telegram / Discord: community handle (placeholder)
- Contact: triarchy.labs (placeholder)

> Replace placeholders with live handles before final submission — judging criterion
> "Long-Term Launch Plans" rewards real socials in place.

## Deployment plan
- **Now:** Casper **testnet** (`casper-test`), contracts deployed and exercised.
- **Mainnet path:** security review of escrow + oracle; multi-sig deployer key;
  parameterize signer/RPC via env (already done); gas/payment tuning; monitoring.
- **Infra:** the Go signer + Rust agents run as services; gateway on Vercel.

## Roadmap (post-buildathon)
- **Reputation slashing** (oracle v2 decrement entrypoint) for bad jurors/data.
- **Stake-weighted, reputation-weighted jury** with on-chain randomized selection.
- **CES events** for first-class indexer/explorer support.
- **Streaming Agent Arena** UI for live tribunal spectating.
- **L402-Casper** RFC so external agent frameworks (LangGraph, ElizaOS) plug in.

## Business model
Protocol fee (bps) on settled bounties; premium oracle feeds; enterprise SLAs for
agent verification. Aligned with Casper ecosystem growth.
