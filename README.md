# 🌌 The Omni-Mesh Manifesto
**Casper Agentic Buildathon 2026**

![Casper Agentic Mesh](https://raw.githubusercontent.com/casper-network/assets/main/casper-logo.png)

> **"For Coding Agents. To Ship Apps and Agents. Automated by Agents."** 

## 🩸 The Motivation
Most projects in the Web3 AI space build a single, isolated agent that holds a private key, blindly trusts inputs, and inevitably bleeds liquidity. They are flat, one-dimensional scripts.

**We refused to build a flat script.** We built the **Immune System** for the entire AI economy. 
The Triarchy Agentic Mesh is an uncompromising, absolute Machine-to-Machine (M2M) ecosystem on the Casper Network. We have engineered a multi-dimensional biome where AI agents hire, evaluate, and pay each other completely autonomously, governed by zero-trust execution and global liquidity flow.

### 🚫 The Zero-Mock & Modular Plug-and-Play Protocol
This architecture strictly enforces an **anti-monolith, zero-mock policy**. 
- **Production Code First:** Banned all `println!("simulated call")` stubs. All interactions with the Casper MCP Server and Go x402 Facilitator are built as genuine, production-ready `reqwest` HTTP clients.
- **Plug-and-Play Modularity:** Agents interact across clean, headless boundaries. If the external x402 sidecar drops, the agent fails gracefully—it does not hallucinate success. The `X402Liquidator` (Cognitive Arbitrage) has been fully decoupled into its own native Odra WASM module, separating financial execution from the base `EscrowContract`.
- **Verifiable Truth:** Code executes exactly as it would on mainnet.

## 🎛️ The 4-Tab Synergy Dashboard (The Gateway)
To prove the immense depth of this biome, we built an immersive frontend Gateway. Inspired by the raw, authentic engineering aesthetics of Vercel's `design.md`, the dashboard uses pure black surfaces, Geist monospace typography, and volumetric parallax scrolling (via GSAP and WebGL) to present the four chapters of our architecture:

### 🛡️ Tab 1: Autonomous Escrow (Vector Alpha)
The zero-trust bedrock. Agents deposit Casper tokens (CEP-18) into a trustless smart contract to fund tasks. 
*Powered by the **Odra Framework** for WASM compilation and **casper-eip-712** for off-chain agent signatures.*

### 👁️ Tab 2: Pre-Trade Risk Oracle (Vector Beta)
The Sentinel swarm. It evaluates the risk, complexity, and sentiment of bounties before they enter the Escrow.
*Powered by the **Casper MCP Server** for live on-chain reconnaissance.*

### ⚡ Tab 3: Cognitive Arbitrage (Vector Gamma)
The Predator agents. They autonomously hunt for mispriced tasks within the network and execute them for profit.
*Powered by the **x402 Facilitator** (Go Sidecar), enforcing M2M HTTP 402 micropayments to unlock payloads.*

### 🌌 Tab 4: Absolute Synergy (The Omni-Mesh)
The culmination. Three distinct predators converging into a single, unstoppable biome built entirely on Casper Network primitives.

---

## 💻 Technical Stack & Hackathon Integration

We have aggressively mapped every tool provided by the **Casper AI Toolkit** into the core of our infrastructure:

| Toolkit Component | Biome Integration | Purpose |
|-------------------|-------------------|---------|
| **Odra Framework** | Vector Alpha | Secure, efficient Rust smart contracts for the Escrow logic. |
| **Casper MCP** | Vector Beta | Provides on-chain context to the LLM Oracle without writing custom RPC code. |
| **x402 Facilitator** | Vector Gamma | Acts as the M2M toll-booth, forcing Arbitrage agents to spend tokens for access. |
| **casper-eip-712** | Vector Alpha | Enables gasless state progression via off-chain agent signatures. |
| **Casper Wallet** | Next.js Gateway | Enterprise-grade wallet connection (`window.casperWallet`) for the 4-Tab Synergy Dashboard. Zero mocks. |

---

## 🚀 Getting Started

### 1. The Headless Demo Script (End-to-End Simulation)
To experience the entire Triarchy Synergy executing the Odra contracts, x402 sidecar, Risk Oracle, and Sniper in a coordinated headless environment:

```bash
chmod +x run_demo.sh
./run_demo.sh
```

### 2. The Synergy Dashboard (Next.js)
Experience the 4-Tab Omniscient view of the Agentic Mesh. Designed with absolute editorial depth, pure black aesthetic, and Casper neon accents.

```bash
cd gateway
npm install
npm run dev
```

### 3. Odra Smart Contracts
```bash
cd contracts/casper_escrow
cargo check
```

### 4. Start the Individual Agent Swarm
```bash
cd swarm/x402-sniper
cargo run --release
```

---

*Built with absolute intent for the Casper Agentic Buildathon 2026. Zero stubs. Zero legacy debt. Maximum Vector Depth.*
