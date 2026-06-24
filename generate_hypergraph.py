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
        
    # 3. Swarm Crates
    swarm_dir = PROJECT_DIR / "swarm"
    for crate in swarm_dir.iterdir():
        if crate.is_dir() and crate.name != "casper-client":
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
        f.write("> **Автоматически сгенерированный интерактивный индекс связей и структуры проекта.**\n\n")
        
        f.write("## 🗺️ 1. Архитектурный Гиперграф Связей (Mermaid)\n\n")
        f.write("```mermaid\ngraph TD\n")
        
        # Define Nodes
        f.write("    subgraph UI[\"🖥️ GATEWAY FRONTEND & DESKTOP (Tauri/Next.js)\"]\n")
        f.write("        Dashboard[\"Dashboard Component\"] -->|IPC calls| TauriBridge[\"Tauri Host Commands\"]\n")
        f.write("        BountiesView[\"Bounties Panel Component\"] -->|Escrow tracking| StateMonitor[\"Zustand State Store\"]\n")
        f.write("    end\n\n")
        
        f.write("    subgraph ROYS[\"🐝 OFF-CHAIN AGENTIC ROYS (Rust)\"]\n")
        f.write("        Engine[\"swarm-engine (Ciel Swarm Orchestrator)\"] -->|Decision loop| Brain[\"ouroboros-brain (LLM Multi-Agent Debate)\"]\n")
        f.write("        Engine -->|Micropayment assertions| Consensus[\"x402-consensus (2PC State Validation)\"]\n")
        f.write("        Consensus -->|Risk check| Risk[\"x402-risk (15-Factor Judge)\"]\n")
        f.write("        Engine -->|Agent memory vectors| Memory[\"x402-memory (Sovereign Vector DB)\"]\n")
        f.write("    end\n\n")
        
        f.write("    subgraph CASPER[\"⛓️ CASPER ON-CHAIN MESH NETWORK\"]\n")
        f.write("        Client[\"casper-client (Rust RPC Transaction Signer)\"] -->|JSON-RPC calls| Node[\"Casper Node RPC\"]\n")
        f.write("        Node -->|Triggers EntryPoints| Contract[\"casper-mesh-contract (WASM Smart Contract)\"]\n")
        f.write("        Contract -->|State updates| AgentRegistry[\"Agent Entity Registry\"]\n")
        f.write("        Contract -->|State updates| EscrowStore[\"Micropayment Escrow Vault\"]\n")
        f.write("    end\n\n")
        
        # Connect Components
        f.write("    %% Connections between layers\n")
        f.write("    TauriBridge -->|RPC over Localhost| Engine\n")
        f.write("    Engine -->|Signs txns via Client| Client\n")
        f.write("    Dashboard -->|Read balances/events| Node\n")
        
        f.write("    %% Styles\n")
        f.write("    style Contract fill:#ff5e5e,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("    style Engine fill:#4a90e2,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("    style Client fill:#f5a623,stroke:#fff,stroke-width:2px,color:#fff\n")
        f.write("    style Dashboard fill:#7ed321,stroke:#fff,stroke-width:2px,color:#fff\n")
        
        f.write("```\n\n")
        
        f.write("## 📝 2. Реестр Модулей и Символов AST (Спецификация)\n\n")
        
        total_symbols = sum(len(syms) for syms in symbols.values())
        f.write(f"Всего проиндексировано **{len(symbols)} модулей** и **{total_symbols} ключевых символов (AST definitions)**.\n\n")
        
        for module_name, syms in symbols.items():
            f.write(f"### 📂 {module_name}\n\n")
            f.write("| Тип | Имя символа | Путь к файлу |\n")
            f.write("| :--- | :--- | :--- |\n")
            for s in sorted(syms, key=lambda x: (x["kind"], x["name"])):
                f.write(f"| `{s['kind']}` | `{s['name']}` | `{s['file']}` |\n")
            f.write("\n---\n\n")
            
    print(f"✅ AST Hypergraph report generated successfully: {report_path}")

if __name__ == "__main__":
    syms = scan_files()
    generate_report(syms)
