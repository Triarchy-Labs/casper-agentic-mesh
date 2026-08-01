import { NextResponse } from "next/server";
import { runAgent } from "@/lib/agents";

export const dynamic = "force-dynamic";
export const maxDuration = 60;

// Cheap status ping — the dashboard's mesh-latency grid GETs every endpoint every 20s.
// Answer without spawning the binary: the scan itself stays click-triggered via POST.
export async function GET() {
	return NextResponse.json({ ok: true, role: "tower-overseer", scan: "POST /api/tower" });
}

// The Tower overseer scan — read-only, click-triggered.
// Local/VM: spawns the compiled binary for a fresh scan. Serverless: proxies to the
// 24/7 overseer on Render (TOWER_SERVICE_URL) — same binary, same chain, running live.
export async function POST() {
	const run = await runAgent("tower", [], 60_000);
	if (run.ok || run.output) {
		return NextResponse.json({ ok: run.ok, lines: run.output.split("\n") });
	}

	const remote = process.env.TOWER_SERVICE_URL;
	if (remote) {
		try {
			const r = await fetch(remote, { cache: "no-store", signal: AbortSignal.timeout(50_000) });
			if (r.ok) {
				const text = await r.text();
				return NextResponse.json({ ok: true, lines: text.split("\n") });
			}
		} catch { /* fall through to the honest error */ }
	}

	return NextResponse.json(
		{ ok: false, error: run.error || "Tower unavailable — server functions are frozen, we are working on it." },
		{ status: 502 },
	);
}
