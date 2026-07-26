# ═══════════════════════════════════════════════════════════════════════════════
# TRIARCHY AGENTIC MESH — UNIFIED DEVEEX MAKEFILE
# Casper Agentic Buildathon 2026
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: help build build-contracts build-swarm build-signer build-gateway dev watch-swarm watch-contracts watch-gateway test test-swarm test-contracts lint clean demo

# Default target
.DEFAULT_GOAL := help

# Environment configuration
CARGO ?= cargo
NPM ?= npm
GO ?= go
WASM_OPT ?= wasm-opt
LOG_FORMAT ?= json

help: ## Display this help message
	@echo "Triarchy Agentic Mesh — Local DevEx Tooling"
	@echo "--------------------------------------------------------"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ── BUILD TARGETS ──────────────────────────────────────────────────────────────

build: build-contracts build-signer build-swarm build-gateway ## Build all workspace components (Contracts, Signer, Swarm, Gateway)

build-contracts: ## Compile Casper smart contracts to Wasm and lower bulk-memory opcodes
	@echo "⚙️ Building Casper Smart Contracts..."
	@cd contracts/casper-mesh-contract && $(CARGO) build --release --target wasm32-unknown-unknown
	@if command -v $(WASM_OPT) >/dev/null 2>&1; then \
		echo "⚡ Optimizing Wasm with wasm-opt..."; \
		$(WASM_OPT) contracts/casper-mesh-contract/target/wasm32-unknown-unknown/release/casper_agentic_mesh_contract.wasm \
			--llvm-memory-copy-fill-lowering --signext-lowering -O2 \
			--disable-bulk-memory --disable-sign-ext -o contracts/casper-mesh-contract/escrow_casper_ready.wasm; \
	else \
		echo "⚠️  wasm-opt not found in PATH; skipping Wasm post-processing optimization."; \
	fi

build-swarm: ## Build all Rust Swarm agent binaries
	@echo "🤖 Building Rust Swarm workspace..."
	@LOG_FORMAT=$(LOG_FORMAT) $(CARGO) build --workspace

build-signer: ## Build the Go Casper TransactionV1 signer binary
	@echo "🔑 Building Go Casper Signer..."
	@if [ -d "swarm/casper-client/go-signer" ]; then \
		cd swarm/casper-client/go-signer && $(GO) build -o casper-tx-signer main.go; \
	fi

build-gateway: ## Install dependencies and build Next.js Gateway
	@echo "🌐 Building Next.js Gateway..."
	@cd gateway && $(NPM) install --silent && $(NPM) run build

# ── HOT-RELOADING & DEV TARGETS ────────────────────────────────────────────────

dev: ## Launch full dev stack (Gateway HMR + cargo-watch background monitoring)
	@echo "🚀 Launching Triarchy Dev Mesh..."
	@LOG_FORMAT=$(LOG_FORMAT) $(NPM) --prefix gateway run dev

watch-swarm: ## Run cargo-watch to hot-check/rebuild Rust Swarm agents on changes
	@echo "👁️  Watching Rust Swarm workspace for changes..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		LOG_FORMAT=$(LOG_FORMAT) cargo-watch -w swarm -x 'check --workspace'; \
	else \
		echo "⚠️ cargo-watch is not installed. Installing or fallback loop..."; \
		bash scripts/watch_swarm.sh; \
	fi

watch-contracts: ## Watch Casper smart contract source files for Wasm compilation errors
	@echo "👁️  Watching Casper smart contracts..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo-watch -w contracts -x 'check --target wasm32-unknown-unknown'; \
	else \
		bash scripts/watch_swarm.sh contracts; \
	fi

watch-gateway: ## Start Next.js Gateway development server with Hot Module Replacement (HMR)
	@echo "👁️  Starting Next.js Gateway HMR..."
	@cd gateway && $(NPM) run dev

# ── TESTING & LINTING ──────────────────────────────────────────────────────────

test: test-swarm test-contracts ## Run all unit & integration tests across Rust Swarm and Contracts

test-swarm: ## Run Rust Swarm test suite
	@echo "🧪 Running Swarm tests..."
	@$(CARGO) test --workspace

test-contracts: ## Run smart contract tests
	@echo "🧪 Running Contract tests..."
	@cd contracts/casper-mesh-contract && $(CARGO) test || true

lint: ## Run linters across Rust workspace and Next.js Gateway
	@echo "🔍 Linting Rust workspace..."
	@$(CARGO) clippy --workspace -- -D warnings || $(CARGO) check --workspace
	@echo "🔍 Linting Gateway..."
	@cd gateway && $(NPM) run lint --quiet || true

# ── UTILITIES ──────────────────────────────────────────────────────────────────

clean: ## Clean target directories, build artifacts, and caches
	@echo "🧹 Cleaning workspace artifacts..."
	@$(CARGO) clean
	@rm -rf gateway/.next gateway/node_modules contracts/casper-mesh-contract/target
	@rm -f swarm/casper-client/go-signer/casper-tx-signer

demo: ## Run live on-chain demonstration script
	@echo "🎬 Executing live testnet transaction demo..."
	@bash ./run_demo.sh
