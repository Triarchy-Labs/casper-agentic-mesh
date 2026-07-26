# Triarchy Boost Layer — edge code over the official Casper AI Toolkit

> Our original edge layer that sits **on top of** the official Casper AI Toolkit and wires it
> into the mesh. We claim authorship only of this Boost Layer and the mesh; the upstream toolkit
> repos are the ecosystem's work, credited below.

The Buildathon ships an [AI Toolkit](https://www.casper.network/ai) — x402, MCP, Odra, CSPR.cloud,
EIP-712 — and asks builders to use it. We do, with attribution, and add a thin, strictly-typed
TypeScript/Rust bridge so those tools plug into a Next.js edge frontend without heavy binaries in
the browser.

| Capability | Official upstream (credited) | Boost Layer (our edge code) | Where it shows in the app |
| :--- | :--- | :--- | :--- |
| **x402 / L402** | [make-software/casper-x402](https://github.com/make-software/casper-x402) (Go facilitator) | `x402_middleware` — a Next.js/Express wrapper | **LIVE**: the gateway's `/api/hire` returns a real `402` until a CSPR payment is verified on the ledger; the dashboard's L402 console fires it live |
| **MCP** | [msanlisavas/casper-mcp](https://github.com/msanlisavas/casper-mcp) (C# server, 98 tools — our `StringBuilder` / `CancellationToken` optimization upstreamed) | `mcp_server` — a TypeScript `@modelcontextprotocol/sdk` server with L402-validation tools and a **failover RPC pool** (`ResilientCasperClient`, 3 nodes) | **LIVE**: `/api/mcp` serves the discovery manifest; the ONE CLICK assistant hits it for real |
| **CSPR.cloud** | official CSPR.cloud API | `cspr_cloud` — REST + WebSocket client with a dead-socket heartbeat | **LIVE**: the dashboard's **Casper Live Stream** renders real testnet blocks + deploys via server-side `/api/casper-stream` (key never shipped to the browser) |
| **Odra** | [odra 0.8](https://odra.dev) | `odra_contracts` — batched-write oracle | an Odra 0.8 oracle contract alongside the native `#![no_std]` one |
| **EIP-712** | [casper-ecosystem/casper-eip-712](https://github.com/casper-ecosystem/casper-eip-712) | typed-data signing | cross-verified between the Go signer and the backend |

---

## What each module actually does

### `x402_middleware`
A Next.js edge / Express middleware that speaks HTTP 402. The frontend never needs a Go binary:
a request without a verified `x-l402-txhash` is refused with a real `402 Payment Required`, and a
provided hash is validated against the ledger (recipient + amount, single-use, TTL). The gateway's
`/api/hire` route enforces exactly this today.

### `mcp_server`
A TypeScript MCP server built on the canonical `@modelcontextprotocol/sdk`. It exposes L402
validation tools to any MCP-speaking agent over stdio, and wraps Casper RPC in a
`ResilientCasperClient` — a failover pool across three testnet nodes with backoff, so a single
node's 503 doesn't take the agent down. Input is validated with `zod` before it reaches a tool.
The gateway also publishes an HTTP discovery manifest at `/api/mcp` for browser/agent discovery.

### `cspr_cloud`
A dual-engine client for the official CSPR.cloud API: REST for point reads, WebSocket for the
event stream, with a ping-pong heartbeat that severs zombie sockets. In the app it powers the
**Casper Live Stream** card — real blocks and deploys scrolling in real time (wired via the
server-side `/api/casper-stream` route so the API key stays hidden).

### `odra_contracts`
An Odra 0.8 oracle contract. `set_prices(assets, prices, decimals)` batches up to 50 assets in
one transaction (amortizing the base deploy fee), stores a `PriceData { price, decimals,
last_updated }` per asset so downstream apps can reject stale data, and guards admin transfer
with a two-step `propose_ownership` / `accept_ownership` to avoid bricking the contract.

---

## Discipline

Strictly typed and linted (`cargo clippy`, `tsc --strict`). The MCP server routes `console.log`
to `stderr` so it can never corrupt the JSON-RPC stdio stream. Payment verification is deep: the
raw `Transfer` args are parsed and the `target` is checked against the gateway account with U512
amount comparison — a valid hash from an unrelated transfer cannot spoof an authorization.

## On the roadmap (not yet wired in the UI)

- The assistant surfacing **each** MCP tool call inline as it runs (today it shows a real MCP
  manifest fetch + a real ledger read on send; per-tool live tracing is next).
- The x402 middleware and cspr_cloud WebSocket engine imported directly by the gateway (today the
  gateway implements the equivalent as first-class API routes; the modules are the reference).

> Every "LIVE" row above is verifiable in the running app — see the project [PLAYBOOK](../PLAYBOOK.md).
