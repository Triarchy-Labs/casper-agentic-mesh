# Triarchy Agentic Mesh — Vision & Vector Map

> The economic operating system for the AI agent economy on Casper:
> settlement (escrow), judiciary (Tribunal), credit bureau (reputation),
> data/central-bank (oracle), workforce (agents), and a brain (The Tower).
>
> This file captures **every** vector so nothing is lost. Status legend:
> ✅ live on testnet · 🔨 building now · 🗺️ roadmap.

## 0. The external primitive nobody scoped — Antifragile Mesh 🔨 (NEW)
The buildathon imagined agents that *work*; it never imagined agents that *die*.
- **Proof-of-Liveness:** each agent posts an on-chain heartbeat (reused via the
  oracle `post_reading` as `HEARTBEAT-<agent>` = timestamp). No redeploy needed.
- **Autonomous Succession:** if an agent's heartbeat goes stale, The Tower nominates
  the highest-reputation *live* agent as successor; the Tribunal ratifies the handover.
- **Dead-agent escrow rescue:** open bounties of a dark agent never freeze — duties
  transfer to the successor. The user's "don't fall if one is out" becomes a *systemic*
  property: the mesh is antifragile, on-chain.
- Ties: oracle (heartbeat+reputation) + Tribunal (ratify) + escrow (rescue) + Tower (detect).

## 1. The Tower — Overseer meta-agent 🔨 (flagship)
The brain at the top of the tower. Reads all on-chain events (oracle readings,
reputation, escrow bounties, heartbeats), builds a single world-model, detects
anomalies, and dispatches the right sub-agent. Click-triggered, never an autonomous
background loop. Dispatch defaults to dry-run; on-chain actions need explicit confirm.
- Ties: everything. Uses oracle event log + reputation already live on-chain.

## 2. Delegated agent custody — Casper associated keys 🗺️
A user grants an agent a **weighted, revocable** key on their account: the agent may
spend up to a limit; the human revokes anytime. Native to Casper (key-weight
thresholds), impossible to do natively on EVM. The killer feature for normal users.
- Ties: escrow funding (agent pays from delegated authority instead of a raw purse).

## 3. Self-amending contracts — governance over code 🗺️
Casper upgradable contracts let an agent propose a logic upgrade, but it goes live
only after the **Tribunal approves** it. Code changes become a judicial process.
- Ties: Tribunal + upgradable escrow/oracle.

## 4. ZK-compliance gate (buildathon #4) 🗺️
An agent issues a compliance/KYC attestation off-chain (ZK-friendly); escrow release
is gated by a valid attestation token. Privacy-preserving settlement.
- Ties: escrow + Tribunal.

## 5. Yield-routing agent (buildathon #1) 🗺️
An agent monitors yields and reallocates, using **our on-chain oracle** price feeds
as its market context. Turns a passive wallet into a self-driving portfolio.
- Ties: oracle.

## 6. Portable Agent Credit Score 🗺️ (independent public good)
The oracle `reputation_dict` exposed as a portable "agent credit score" any external
dApp can read. Network effects beyond this project — the blast radius.
- Independent; readable by anyone.

## 7. Verdict Prediction Market 🗺️ (second external bonus)
Before the Tribunal rules, anyone can stake APPROVE/REJECT; the verdict settles the
market, and crowd odds feed back as a *confidence signal* into agent calibration.
A liquid signal layer over agent decisions — "Augur for agent rulings".
- Ties: Tribunal + escrow.

## Live foundation (already on testnet — see DEPLOYMENTS.md)
✅ Escrow contract (full state machine) ✅ RWA oracle (data+identity+reputation+events)
✅ RWA oracle agent ✅ Bounty-judge (LLM→on-chain) ✅ Tribunal (adversarial court,
fault-tolerant) ✅ Gateway with live on-chain panel + RWA-pegged pricing.

## Operating principles (non-negotiable)
- **Click-triggered only.** No autonomous background hammering. Heavy/LLM/spending
  actions run on a button; only cheap reads poll.
- **Graceful degradation.** Partial bench → honest "indicative, not fully precise";
  all agents down → "functions frozen, no funds moved, we're fixing it".
- **Never fake.** No mock hashes, no invented verdicts; on failure we abort honestly.
