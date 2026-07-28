#!/usr/bin/env python3
import os
import re
from pathlib import Path

# Paths to scan
PROJECT_DIR = Path("/home/minimalmod/Triarchy/Agent_Zone/projects/casper-agentic-mesh")

def scan_files():
    symbols = {}
    relations = []
    
    # 1. Smart Contract
    contract_path = PROJECT_DIR / "contracts/casper-mesh-contract/src/lib.rs"
    if contract_path.exists():
        symbols["Smart Contract"] = parse_rust_file(contract_path)
        
    # 2. Swarm Client
    client_path = PROJECT_DIR / "swarm/casper-client/src/main.rs"
    if client_path.exists():
        symbols["Casper RPC Client"] = parse_rust_file(client_path)
        
    # 3. Swarm Crates — only the ones declared in the Cargo workspace, so the report
    #    can never drift onto stale directories left on disk.
    members = set()
    cargo = (PROJECT_DIR / "Cargo.toml").read_text(encoding="utf-8")
    for line in cargo.splitlines():
        line = line.strip().strip(",").strip('"')
        if line.startswith("swarm/"):
            members.add(line.split("/", 1)[1])

    swarm_dir = PROJECT_DIR / "swarm"
    for crate in swarm_dir.iterdir():
        if crate.is_dir() and crate.name != "casper-client" and crate.name in members:
            crate_symbols = []
            src_dir = crate / "src"
            if src_dir.exists():
                for rs_file in src_dir.glob("**/*.rs"):
                    crate_symbols.extend(parse_rust_file(rs_file))
            if crate_symbols:
                symbols[f"Crate: {crate.name}"] = crate_symbols
                
    # 4. Frontend Next.js and Tauri
    gateway_src = PROJECT_DIR / "gateway/src"
    if gateway_src.exists():
        frontend_symbols = []
        for ts_file in gateway_src.glob("**/*.ts*"):
            if "node_modules" not in str(ts_file) and ".next" not in str(ts_file):
                frontend_symbols.extend(parse_ts_file(ts_file))
        if frontend_symbols:
            symbols["Frontend UI (Next.js)"] = frontend_symbols
            
    # Tauri backend
    tauri_src = PROJECT_DIR / "gateway/src-tauri/src"
    if tauri_src.exists():
        tauri_symbols = []
        for rs_file in tauri_src.glob("**/*.rs"):
            tauri_symbols.extend(parse_rust_file(rs_file))
        if tauri_symbols:
            symbols["Tauri Host Backend"] = tauri_symbols
            
    return symbols

def parse_rust_file(path):
    rel_path = path.relative_to(PROJECT_DIR)
    symbols = []
    try:
        content = path.read_text(errors='ignore')
        # Simple extraction of structs, enums, impls, fns
        structs = re.findall(r'(pub\s+)?struct\s+(\w+)', content)
        enums = re.findall(r'(pub\s+)?enum\s+(\w+)', content)
        impls = re.findall(r'impl\s+(\w+)', content)
        fns = re.findall(r'(pub\s+)?(async\s+)?fn\s+(\w+)', content)
        
        for _, name in structs:
            symbols.append({"kind": "struct", "name": name, "file": str(rel_path)})
        for _, name in enums:
            symbols.append({"kind": "enum", "name": name, "file": str(rel_path)})
        for name in impls:
            symbols.append({"kind": "impl", "name": name, "file": str(rel_path)})
        for _, _, name in fns:
            if name not in ["main", "panic"]:
                symbols.append({"kind": "function", "name": name, "file": str(rel_path)})
    except Exception as e:
        print(f"Error parsing {path}: {e}")
    return symbols

def parse_ts_file(path):
    rel_path = path.relative_to(PROJECT_DIR)
    symbols = []
    try:
        content = path.read_text(errors='ignore')
        interfaces = re.findall(r'export\s+interface\s+(\w+)', content)
        types = re.findall(r'export\s+type\s+(\w+)', content)
        fns = re.findall(r'export\s+(const|function)\s+(\w+)', content)
        
        for name in interfaces:
            symbols.append({"kind": "interface", "name": name, "file": str(rel_path)})
        for name in types:
            symbols.append({"kind": "type", "name": name, "file": str(rel_path)})
        for _, name in fns:
            symbols.append({"kind": "function", "name": name, "file": str(rel_path)})
    except Exception as e:
        print(f"Error parsing {path}: {e}")
    return symbols

def generate_report(symbols):
    report_path = PROJECT_DIR / "AST_HYPERGRAPH.md"
    
    with open(report_path, "w") as f:
        f.write("# 🌌 CASPER AGENTIC MESH — AST HYPERGRAPH & SYSTEM BLUEPRINT\n\n")
        f.write("> **Auto-generated index of the project's structure and symbol graph.**\n\n")
        
        f.write("## 🗺️ 1. Architecture hypergraph (Mermaid)\n\n")
        f.write("```mermaid\ngraph TD\n")
        
        # Define Nodes
        f.write("    subgraph UI[\"🖥️ GATEWAY (Next.js on Vercel)\"]\n")
        f.write("        Home[\"Home · vector dossiers\"] -->|live ledger read| ApiOnchain\n")
        f.write("        Dash[\"Dashboard · Mesh Control\"] -->|click-triggered| ApiTower\n")
        f.write("        Dash --> ApiTribunal\n")
        f.write("        Hub[\"ONE CLICK hub · Try it live\"] --> ApiMcp\n")
        f.write("        Hub --> ApiHire\n")
        f.write("        Wallet[\"Casper Wallet connect\"] -->|signs transfer| ApiHire\n")
        f.write("    end\n\n")

        f.write("    subgraph API[\"🔌 GATEWAY API (server-side keys)\"]\n")
        f.write("        ApiOnchain[\"/api/onchain\"]\n")
        f.write("        ApiHire[\"/api/hire · x402 gate\"]\n")
        f.write("        ApiMcp[\"/api/mcp · MCP manifest\"]\n")
        f.write("        ApiStream[\"/api/casper-stream · CSPR.cloud\"]\n")
        f.write("        ApiTower[\"/api/tower\"]\n")
        f.write("        ApiTribunal[\"/api/tribunal · dry-run\"]\n")
        f.write("    end\n\n")

        f.write("    subgraph AGENTS[\"🐝 AGENT SWARM (compiled Rust)\"]\n")
        f.write("        Tower[\"tower-overseer · proof-of-liveness\"]\n")
        f.write("        Tribunal[\"tribunal · 5-LLM adversarial court\"]\n")
        f.write("        Judge[\"bounty-judge · autonomous payout\"]\n")
        f.write("        Oracle[\"rwa-oracle · CSPR/USD feed\"]\n")
        f.write("    end\n\n")

        f.write("    subgraph CASPER[\"⛓️ CASPER TESTNET (WASM)\"]\n")
        f.write("        Client[\"casper-client · TransactionV1 signer\"] -->|JSON-RPC| Node[\"Casper node\"]\n")
        f.write("        Node -->|entry points| Contract[\"escrow contract\"]\n")
        f.write("        Node --> OracleContract[\"oracle contract\"]\n")
        f.write("        Contract -->|release / refund| Escrow[\"escrow purse\"]\n")
        f.write("    end\n\n")

        f.write("    %% cross-layer\n")
        f.write("    ApiTower --> Tower\n")
        f.write("    ApiTribunal --> Tribunal\n")
        f.write("    ApiOnchain --> Node\n")
        f.write("    ApiStream -->|official API| Cloud[\"CSPR.cloud\"]\n")
        f.write("    Tribunal -->|verdict moves CSPR| Client\n")
        f.write("    Judge -->|release_bounty| Client\n")
        f.write("    Oracle -->|price post| Client\n")
        f.write("    Tower -->|read-only scan| Node\n\n")

        f.write("    style Contract fill:#ff5e5e,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("    style Tribunal fill:#4a90e2,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("    style Client fill:#f5a623,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("    style Hub fill:#7ed321,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("```\n\n")
        
        f.write("## 📝 2. Module & AST symbol registry\n\n")
        
        total_symbols = sum(len(syms) for syms in symbols.values())
        f.write(f"Indexed **{len(symbols)} modules** and **{total_symbols} AST definitions**.\n\n")
        
        for module_name, syms in symbols.items():
            f.write(f"### 📂 {module_name}\n\n")
            f.write("| Kind | Symbol | File |\n")
            f.write("| :--- | :--- | :--- |\n")
            for s in sorted(syms, key=lambda x: (x["kind"], x["name"])):
                f.write(f"| `{s['kind']}` | `{s['name']}` | `{s['file']}` |\n")
            f.write("\n---\n\n")
            
    print(f"✅ AST Hypergraph report generated successfully: {report_path}")

if __name__ == "__main__":
    syms = scan_files()
    generate_report(syms)
