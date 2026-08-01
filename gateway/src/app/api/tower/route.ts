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
export async function POST() {
	const run = await runAgent("tower", [], 60_000);
	if (!run.ok && !run.output) {
		return NextResponse.json(
			{ ok: false, error: run.error || "Tower unavailable — server functions are frozen, we are working on it." },
			{ status: 502 },
		);
	}
	return NextResponse.json({ ok: run.ok, lines: run.output.split("\n") });
}
