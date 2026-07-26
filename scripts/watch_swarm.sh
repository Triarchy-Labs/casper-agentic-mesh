#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# TRIARCHY CARGO-WATCH SCRIPT
# Hot-reloading watcher script with cargo-watch integration & fallback loop
# ═══════════════════════════════════════════════════════════════════════════════

set -e

MODE="${1:-swarm}"
LOG_FORMAT="${LOG_FORMAT:-json}"

export LOG_FORMAT

echo "👁️  [DevEx Watcher] Mode: ${MODE} | Log Format: ${LOG_FORMAT}"

if command -v cargo-watch >/dev/null 2>&1; then
    if [ "$MODE" = "contracts" ]; then
        echo "🚀 Running cargo-watch for smart contracts..."
        cargo watch -w contracts -x 'check --target wasm32-unknown-unknown'
    else
        echo "🚀 Running cargo-watch for Rust Swarm workspace..."
        cargo watch -w swarm -x 'check --workspace'
    fi
else
    echo "⚠️  cargo-watch tool not detected in PATH."
    echo "💡 Install via 'cargo install cargo-watch' for optimal low-latency HMR."
    echo "🔄 Falling back to polling watcher loop (checking every 3s)..."
    
    LAST_HASH=""
    while true; do
        if [ "$MODE" = "contracts" ]; then
            CURRENT_HASH=$(find contracts -type f -name "*.rs" -exec md5sum {} + 2>/dev/null | sort | md5sum)
            CHECK_CMD="cargo check --target wasm32-unknown-unknown --manifest-path contracts/casper-mesh-contract/Cargo.toml"
        else
            CURRENT_HASH=$(find swarm -type f -name "*.rs" -exec md5sum {} + 2>/dev/null | sort | md5sum)
            CHECK_CMD="cargo check --workspace"
        fi

        if [ "$CURRENT_HASH" != "$LAST_HASH" ]; then
            if [ -n "$LAST_HASH" ]; then
                echo "⚡ [Watcher] Code change detected! Rechecking..."
                eval "$CHECK_CMD" || echo "❌ Build check failed."
            else
                echo "✅ Initial check completed."
                eval "$CHECK_CMD" || true
            fi
            LAST_HASH="$CURRENT_HASH"
        fi
        sleep 3
    done
fi
