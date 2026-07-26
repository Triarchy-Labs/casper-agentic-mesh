import { NextResponse } from "next/server";

export const revalidate = 0;

// LIVE Casper network feed via the official CSPR.cloud API (Buildathon AI Toolkit).
// The key stays server-side (never shipped to the browser). Returns the latest blocks and
// deploys, formatted as a scrolling matrix of REAL on-chain events — no synthetic data.
const BASE = "https://api.testnet.cspr.cloud";

async function cc(path: string, key: string) {
	const ctrl = new AbortController();
	const to = setTimeout(() => ctrl.abort(), 6000);
	try {
		const r = await fetch(`${BASE}${path}`, {
			headers: { Authorization: key },
			signal: ctrl.signal,
			cache: "no-store",
		});
		clearTimeout(to);
		if (!r.ok) return null;
		return await r.json();
	} catch {
		clearTimeout(to);
		return null;
	}
}

export async function GET() {
	const key = process.env.CSPR_CLOUD_API_KEY;
	if (!key) {
		return NextResponse.json(
			{ ok: false, source: "cspr.cloud", error: "stream key not configured", lines: [] },
			{ status: 200 },
		);
	}

	const [blocks, deploys] = await Promise.all([
		cc("/blocks?page=1&page_size=6", key),
		cc("/deploys?page=1&page_size=6", key),
	]);

	if (!blocks) {
		return NextResponse.json(
			{ ok: false, source: "cspr.cloud", error: "network degraded — feed frozen, no data faked", lines: [] },
			{ status: 200 },
		);
	}

	const lines: string[] = [];
	for (const b of blocks.data ?? []) {
		const h = b.block_height ?? "?";
		const era = b.era_id ?? "?";
		const calls = b.contract_calls_number ?? 0;
		lines.push(`[BLOCK ${h}] era ${era} · ${calls} contract-call${calls === 1 ? "" : "s"} · gas ${b.gas_price ?? 1} · OK`);
	}
	for (const d of (deploys?.data ?? []).slice(0, 4)) {
		const hash = (d.deploy_hash ?? d.transaction_hash ?? "").slice(0, 10);
		const status = d.error_message ? "ERR" : "OK";
		lines.push(`[DEPLOY ${hash}…] ${status}`);
	}

	// interleave blocks and deploys for a lively matrix, newest first
	return NextResponse.json({
		ok: true,
		source: "cspr.cloud · testnet · live",
		height: blocks.data?.[0]?.block_height ?? null,
		total_blocks: blocks.item_count ?? null,
		lines,
	});
}
