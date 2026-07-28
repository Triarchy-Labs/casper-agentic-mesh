# Contributing

Thanks for looking at Triarchy Agentic Mesh. This is a Casper Agentic Buildathon 2026 project;
issues and pull requests are welcome.

## Ground rules

- **Every claim must be verifiable.** If you add a feature that touches the chain, include the
  transaction hash (testnet) so a reader can open it on [testnet.cspr.live](https://testnet.cspr.live).
  We never ship a mock behind language that implies live data — if a feed is down, the UI says so.
- **No secrets in commits.** The only key in this repo is a disposable, faucet-funded testnet key
  documented in [SECURITY.md](SECURITY.md). Everything else lives in `.env.local`.
- **Attribute upstream work.** The official Casper AI Toolkit components we integrate are credited
  in the README; keep it that way when you extend them.

## Getting set up

```bash
# Rust agents + contracts
cargo build --workspace
cargo test --workspace

# Gateway
cd gateway && npm install && npm run dev
```

Reproduce the on-chain lifecycle end to end with [PLAYBOOK.md](PLAYBOOK.md) — it runs in about
ten minutes and needs no wallet for the browser path.

## Pull requests

1. Branch off `main`.
2. Keep `cargo test --workspace`, `cargo build --workspace` and `npm run build` (in `gateway`) green.
3. Describe *what changed and why* — the reasoning matters more than the diff.
4. If the change alters behaviour a judge or user can see, update the docs in the same PR.

## Reporting a vulnerability

See [SECURITY.md](SECURITY.md). Please do not open a public issue for a security problem.
