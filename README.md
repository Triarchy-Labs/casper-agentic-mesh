◢◤￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣￣◥◣

            the omni-mesh manifesto
         casper agentic buildathon 2026

◥◣＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿＿◢◤

"for coding agents. to ship apps and agents. automated by agents."

────────────────────────────────────────────────────────────────

// 𝟎𝟏 // the motivation

most projects in the web3 ai space build a single, isolated agent that holds a private key, blindly trusts inputs, and inevitably bleeds liquidity. they are flat, one-dimensional scripts.

we refused to build a flat script. we built the **immune system** for the entire ai economy. 

the triarchy agentic mesh is an uncompromising, absolute machine-to-machine (m2m) ecosystem on the casper network. we have engineered a multi-dimensional biome where ai agents hire, evaluate, and pay each other completely autonomously, governed by zero-trust execution and global liquidity flow.

────────────────────────────────────────────────────────────────

// 𝟎𝟐 // the zero-mock & modular plug-and-play protocol

this architecture strictly enforces an **anti-monolith, zero-mock policy**.

◆ **production code first:** banned all `println!("simulated call")` stubs. all interactions with the casper mcp server and go x402 facilitator are built as genuine, production-ready `reqwest` http clients.
◆ **plug-and-play modularity:** agents interact across clean, headless boundaries. if the external x402 sidecar drops, the agent fails gracefully—it does not hallucinate success. the `x402liquidator` (cognitive arbitrage) has been fully decoupled into its own native odra wasm module, separating financial execution from the base `escrowcontract`.
◆ **verifiable truth:** code executes exactly as it would on mainnet.

────────────────────────────────────────────────────────────────

// 𝟎𝟑 // the 4-tab synergy dashboard (the gateway)

to prove the immense depth of this biome, we built an immersive frontend gateway. evolving beyond conventional ui, we instituted the **cinematic brutalism** design standard: pure black `.editorial-panel` surfaces, geist `monospace` typography, 12-column css grid layouts, and buttery-smooth `gsap scrolltrigger` + `lenis` integrations to present the four chapters of our architecture:

◆ **tab 1: autonomous escrow (vector alpha)**
  ↳ the zero-trust bedrock. agents deposit casper tokens (cep-18) into a trustless smart contract to fund tasks.
  ↳ *powered by the **odra framework** for wasm compilation and **casper-eip-712** for off-chain agent signatures.*

◆ **tab 2: pre-trade risk oracle (vector beta)**
  ↳ the sentinel swarm. it evaluates the risk, complexity, and sentiment of bounties before they enter the escrow.
  ↳ *powered by the **casper mcp server** for live on-chain reconnaissance.*

◆ **tab 3: cognitive arbitrage (vector gamma)**
  ↳ the predator agents. they autonomously hunt for mispriced tasks within the network and execute them for profit.
  ↳ *powered by the **x402 facilitator** (go sidecar), enforcing m2m http 402 micropayments to unlock payloads.*

◆ **tab 4: absolute synergy (the omni-mesh)**
  ↳ the culmination. three distinct predators converging into a single, unstoppable biome built entirely on casper network primitives.

────────────────────────────────────────────────────────────────

// 𝟎𝟒 // technical stack & hackathon integration

we have aggressively mapped every tool provided by the **casper ai toolkit** into the core of our infrastructure:

| toolkit component | biome integration | purpose |
|-------------------|-------------------|---------|
| **odra framework** | vector alpha | secure, efficient rust smart contracts for the escrow logic. |
| **casper mcp** | vector beta | provides on-chain context to the llm oracle without writing custom rpc code. |
| **x402 facilitator** | vector gamma | acts as the m2m toll-booth, forcing arbitrage agents to spend tokens for access. |
| **casper-eip-712** | vector alpha | enables gasless state progression via off-chain agent signatures. |
| **casper wallet** | next.js gateway | enterprise-grade wallet connection (`window.casperwallet`) for the 4-tab synergy dashboard. zero mocks. |

────────────────────────────────────────────────────────────────

// 𝟎𝟓 // getting started

### 1. the headless demo script (end-to-end simulation)
to experience the entire triarchy synergy executing the odra contracts, x402 sidecar, risk oracle, and sniper in a coordinated headless environment:

```bash
chmod +x run_demo.sh
./run_demo.sh
```

### 2. the synergy dashboard (next.js)
experience the 4-tab omniscient view of the agentic mesh. designed with absolute editorial depth, pure black aesthetic, and casper neon accents.

```bash
cd gateway
npm install
npm run dev
```

### 3. odra smart contracts
```bash
cd contracts/casper_escrow
cargo check
```

### 4. start the individual agent swarm
```bash
cd swarm/x402-sniper
cargo run --release
```

────────────────────────────────────────────────────────────────

*built with absolute intent for the casper agentic buildathon 2026. zero stubs. zero legacy debt. maximum vector depth.*

// triarchy labs
