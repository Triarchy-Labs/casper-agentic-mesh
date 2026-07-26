import { NextResponse } from "next/server";
import { runAgent } from "@/lib/agents";

export const dynamic = "force-dynamic";
export const maxDuration = 300;

// Convene the Tribunal in DRY-RUN (deliberation only — no funds moved).
// A judge may supply their own OpenRouter key per request (BYO-key). It is used
// once for the spawned process env and never stored or logged.
export async function POST(req: Request) {
	let body: { taskId?: string; description?: string; proof?: string; apiKey?: string } = {};
	try { body = await req.json(); } catch { /* defaults */ }

	const judgeKey = typeof body.apiKey === "string" && body.apiKey.trim().startsWith("sk-") ? body.apiKey.trim() : undefined;
	if (!process.env.OPENROUTER_API_KEY && !judgeKey) {
		return NextResponse.json(
			{ ok: false, error: "Tribunal offline: no model key on the server. Paste your own OpenRouter key in the panel (used once, never stored) — the court will convene with it." },
			{ status: 503 },
		);
	}

	const args = [
		"--dry-run",
		"--task-id", body.taskId || `ui-${Date.now()}`,
		"--description", body.description || "Deliver the requested work.",
		"--proof", body.proof || "",
	];
	const run = await runAgent("tribunal", args, 280_000, judgeKey ? { OPENROUTER_API_KEY: judgeKey } : undefined);
	if (!run.output) {
		return NextResponse.json(
			{ ok: false, error: run.error || "Tribunal unavailable — please retry shortly." },
			{ status: 502 },
		);
	}
	return NextResponse.json({ ok: run.ok, transcript: run.output });
}
