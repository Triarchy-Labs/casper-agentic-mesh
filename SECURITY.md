# Security Policy

## Reporting a Vulnerability

If you find a vulnerability in any part of this repository (gateway, contracts, swarm agents,
signer), please **do not open a public issue**. Report it privately:

- GitHub: use [Private vulnerability reporting](https://github.com/Triarchy-Labs/casper-agentic-mesh/security/advisories/new)
  on this repository, or
- Email: y4motion@gmail.com with subject `SECURITY: casper-agentic-mesh`.

You will get an acknowledgement within 48 hours. Please include reproduction steps and, where
relevant, the network (Casper testnet) and transaction hashes involved.

## Scope

| Component | In scope |
|---|---|
| `gateway/` (Next.js API routes, x402/L402 payment validation, replay guard) | ✅ |
| `contracts/` (casper-mesh-contract, deposit-proxy, oracle-contract) | ✅ |
| `swarm/` (bounty-judge, casper-client, go-signer) | ✅ |
| Third-party infrastructure (Casper testnet nodes, cspr.live, OpenRouter) | ❌ report upstream |

Everything here runs against **Casper testnet**; no mainnet funds are at risk. The signing key
committed for judging (`swarm/casper-client/key.pem`) is a **disposable testnet key** funded from
the faucet — it is intentionally public so judges can reproduce the on-chain lifecycle, and it
must never be reused outside this demo.

## Hardening already in place

- **Escrow is bounded by construction**: the contract's only money-moving paths are
  `release_bounty` → registered hunter and `refund_bounty` → original creator. No verdict,
  including an LLM verdict, can direct funds anywhere else.
- **Payment validation is on-chain**: `x-l402-txhash` payments are verified against the ledger
  (recipient + amount), with a replay guard on transaction hashes.
- **No request-derived URLs** in server-side fetches (SSRF class removed; internal route calls
  are in-process).
- **No `innerHTML` sinks** for DOM-derived text in the gateway UI.
- The LLM sits **outside the trust path**: it argues and votes, but what it can trigger is
  constrained to the two contract paths above.

## Accepted risks (documented deliberately)

- **`wee_alloc` (RUSTSEC, transitive via `casper-contract`)** — flagged critical by Dependabot,
  dismissed with rationale: it is a compile-time wasm allocator that executes only inside the
  deterministic Casper VM sandbox (no network, no filesystem); the advisory class is
  "unmaintained crate", not a known exploit; and no patched version exists upstream. The deployed
  contract artifacts are frozen for buildathon judging — rebuilding them would change the audited
  artifact hashes without reducing real risk. This will be revisited when `casper-contract`
  drops the dependency.

- **`ed25519-dalek` 1.0.1 (medium), `curve25519-dalek` 3.2.1 and `rand` 0.7.3 (low)** — all three
  reach us through one chain in the Boost Layer's reference contract:
  `odra 0.8.1 → odra-core → casper-event-standard → casper-types 3.0.0`. Patched releases exist,
  but Cargo cannot take them while Odra pins `casper-types` 3.x, and the only route to a newer
  `casper-types` is Odra 2.9 — a rewrite-level jump from 0.8 with a different API. These crates
  sit in a reference contract that is not deployed and holds no funds; the shipped escrow and
  oracle contracts are native `casper-contract` and do not depend on them. Revisit when the Boost
  Layer is migrated to Odra 2.x.

- **`elliptic` ≤ 6.6.1 (low, two manifests)** — the advisory covers *every published version*,
  including the latest; no patched release exists. It arrives transitively under `casper-js-sdk`
  and is used for signature verification against the ledger, where a forged signature still fails
  the on-chain check that follows. Tracked for the day upstream publishes a fix.

All four are below the "high severity or above" bar we hold ourselves to, and none is reachable
in a way that moves funds. We list them rather than silently carrying them.

## Supported Versions

This is a Casper Agentic Buildathon 2026 submission; the `main` branch is the only supported
version. Security fixes land there directly.
