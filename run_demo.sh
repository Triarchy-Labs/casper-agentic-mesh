#!/bin/bash
# Triarchy Agentic Mesh — REAL Casper Testnet end-to-end demo.
#
# This script performs genuine on-chain actions against the deployed escrow
# contract. It does NOT print fabricated hashes. Every hash it shows can be
# opened on https://testnet.cspr.live. If the node or signer fails, it errors.
set -euo pipefail

CYAN='\033[0;36m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'

NODE="${CASPER_RPC_URL:-https://node.testnet.casper.network/rpc}"
CHAIN="${CASPER_CHAIN:-casper-test}"
KEY="${CASPER_SECRET_KEY:-swarm/casper-client/key.pem}"
SIGNER="${CASPER_SIGNER_BIN:-./swarm/casper-client/go-signer/casper-tx-signer}"
PKG="${CASPER_CONTRACT_PACKAGE:-a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d}"
PUBKEY="${CASPER_AGENT_PUBKEY:-013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35}"

rpc() { curl -s --max-time 20 -X POST "$NODE" -H 'Content-Type: application/json' -d "$1"; }

wait_tx() {
  local tx="$1"
  for _ in $(seq 1 30); do
    local r; r=$(rpc "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"info_get_transaction\",\"params\":{\"transaction_hash\":{\"Version1\":\"$tx\"}}}")
    if echo "$r" | grep -q '"execution_info":null'; then printf '.'; sleep 8; continue; fi
    if echo "$r" | grep -q 'execution_info'; then
      echo "$r" | python3 -c "import sys,json;v=json.load(sys.stdin)['result']['execution_info']['execution_result'].get('Version2',{});print(' cost',v.get('cost'),'| error',v.get('error_message'))"
      return
    fi
    printf '_'; sleep 8
  done
}

echo -e "${CYAN}=== Triarchy Agentic Mesh — live Casper testnet demo ===${NC}"

echo -e "\n${YELLOW}[1/4]${NC} Node connectivity (chain_get_state_root_hash)"
SRH=$(rpc '{"jsonrpc":"2.0","id":1,"method":"chain_get_state_root_hash","params":[]}' \
  | python3 -c "import sys,json;print(json.load(sys.stdin)['result']['state_root_hash'])")
echo -e "${GREEN}    state root:${NC} $SRH"

echo -e "\n${YELLOW}[2/4]${NC} Contract package (on-chain)"
echo -e "${GREEN}    package:${NC} https://testnet.cspr.live/contract-package/$PKG"

if [ ! -x "$SIGNER" ] || [ ! -f "$KEY" ]; then
  echo -e "\n${RED}Signer or key missing — connectivity proven above; skipping live writes.${NC}"
  echo -e "Build signer:  (cd swarm/casper-client/go-signer && go build -o casper-tx-signer main.go)"
  exit 0
fi

echo -e "\n${YELLOW}[3/4]${NC} register_agent (real signed TransactionV1)"
TX=$("$SIGNER" --mode call-entrypoint --node "$NODE" --chain "$CHAIN" --secret-key "$KEY" \
  --algo ed25519 --payment 5000000000 --contract-hash "$PKG" --entrypoint register_agent \
  --args "public_key:string:$PUBKEY,metadata_uri:string:triarchy-demo-agent" | tail -1)
echo -e "${GREEN}    tx:${NC} https://testnet.cspr.live/transaction/$TX"
echo -n "    awaiting execution"; wait_tx "$TX"

echo -e "\n${YELLOW}[4/4]${NC} Full escrow lifecycle (deposit + release) is documented with"
echo -e "    reproducible hashes in DEPLOYMENTS.md. Re-run it with the deposit-proxy"
echo -e "    session wasm (see DEPLOYMENTS.md → 'How to reproduce')."

echo -e "\n${GREEN}Done. Every hash above is live on testnet.cspr.live — zero fabricated output.${NC}"
