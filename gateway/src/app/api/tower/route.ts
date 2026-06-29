import { NextResponse } from "next/server";
import { runAgent } from "@/lib/agents";

export const dynamic = "force-dynamic";
export const maxDuration = 60;

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
