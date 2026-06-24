/**
 * MCP TOOL DISCOVERY — Gateway Capability Endpoint
 * 
 * Assimilated from: devloot-mcp-server (AgentFund / Arthur)
 * Pattern: server.tool("question_market", ...) onboarding discovery
 * 
 * Provides a machine-readable capability manifest so external AI agents
 * can discover what tools the x402 gateway offers via MCP protocol.
 * This is the "question_market" onboarding pattern mutated for our gateway.
 */

import { NextResponse } from "next/server";

const GATEWAY_MANIFEST = {
	name: "x402-triarchy-gateway",
	version: "1.0.0",
	description: "Pay-per-use AI compute gateway. Submit USDC, receive AI task results. Casper/Casper native.",
	onboarding: [
		"1. Connect CSPR.click wallet (Casper)",
		"2. Submit USDC payment via Casper transaction",
		"3. POST /api/hire with x-l402-txhash header + task description",
		"4. Receive AI-generated result synchronously or via async delegation",
	],
	data_conventions: {
		amounts: "All prices in USDC (e.g. 5.00 = $5). Minimum $0.01, maximum $10,000 per call.",
		payment: "Casper txHash required. Each hash can only be used once (ReplayGuard: 5min TTL).",
		budget: "Per-caller daily limit: $50,000. Global daily limit: $500,000.",
	},
	tools: {
		compute: {
			hire: {
				method: "POST",
				path: "/api/hire",
				description: "Submit a paid AI task. Routes to optimal executor (Cloud/Local/P2P).",
				headers: ["x-l402-txhash (required)", "x-l402-mode (optional: 'subscription')"],
				body: { description: "string", bounty_usdc: "number", client_id: "string", task_id: "string" },
			},
		},
		bounties: {
			list: { method: "GET", path: "/api/bounties", description: "List all open bounties" },
			create: { method: "POST", path: "/api/bounties", description: "Create a new bounty (requires CSPR.click)" },
		},
		discovery: {
			manifest: { method: "GET", path: "/api/mcp", description: "This endpoint. Machine-readable capability manifest." },
		},
	},
	tiers: {
		micro: { range: "$0.01 - $5.00", executor: "OpenRouter Cloud (claude-3.5-sonnet)", latency: "~2-10s" },
		enterprise: { range: "$5.00+", executor: "Sovereign Enterprise Node", latency: "~5-30s" },
		p2p: { range: "overflow", executor: "Peer-to-Peer Network Mercenary", latency: "~10-60s" },
	},
	security: {
		replay_guard: "5-minute TTL, each txHash single-use",
		spending_policy: "allowlist/blocklist + per-call/daily/global caps",
		ssrf_protection: "isAllowedUrl blocks private subnets on external fetches",
		wasm_sandbox: "Foreign payloads validated via Extism WASI plugin",
	},
	links: {
		docs: "https://github.com/Triarchy-Labs",
		dashboard: "/dashboard",
		bounties: "/bounties",
	},
};

export async function GET() {
	return NextResponse.json(GATEWAY_MANIFEST, {
		status: 200,
		headers: { "Cache-Control": "public, max-age=3600" },
	});
}
