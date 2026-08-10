import { NextResponse } from "next/server";
import { runAgent } from "@/lib/agents";
import fs from "node:fs";
import path from "node:path";

const TRIBUNAL_BIN = path.join(
	process.env.MESH_REPO_ROOT || path.resolve(process.cwd(), ".."),
	"target", "debug", "tribunal",
);

export const dynamic = "force-dynamic";
export const maxDuration = 300;

// Convene the Tribunal in DRY-RUN (deliberation only — no funds moved).
// A judge may supply their own OpenRouter key per request (BYO-key). It is used
// once for the spawned process env and never stored or logged.
export async function POST(req: Request) {
	let body: { taskId?: string; description?: string; proof?: string; apiKey?: string } = {};
	try { body = await req.json(); } catch { /* defaults */ }

	// On serverless the compiled court simply is not there — say so before inviting
	// anyone to paste a key into a panel that cannot use it.
	if (!fs.existsSync(TRIBUNAL_BIN)) {
		return NextResponse.json(
			{ ok: false, error: "🛑 FUNCTIONS FROZEN: the court is a compiled Rust binary and cannot run on serverless. No funds moved, nothing faked — run it in one command, see PLAYBOOK.md." },
			{ status: 502 },
		);
	}

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
