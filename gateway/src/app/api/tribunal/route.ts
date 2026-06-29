import { NextResponse } from "next/server";
import { runAgent } from "@/lib/agents";

export const dynamic = "force-dynamic";
export const maxDuration = 300;

// Convene the Tribunal in DRY-RUN (deliberation only — no funds moved).
export async function POST(req: Request) {
	let body: { taskId?: string; description?: string; proof?: string } = {};
	try { body = await req.json(); } catch { /* defaults */ }

	if (!process.env.OPENROUTER_API_KEY) {
		return NextResponse.json(
			{ ok: false, error: "Tribunal offline: model access not configured on the server. Functions are frozen — we are working on it." },
			{ status: 503 },
		);
	}

	const args = [
		"--dry-run",
		"--task-id", body.taskId || `ui-${Date.now()}`,
		"--description", body.description || "Deliver the requested work.",
		"--proof", body.proof || "",
	];
	const run = await runAgent("tribunal", args, 280_000);
	if (!run.output) {
		return NextResponse.json(
			{ ok: false, error: run.error || "Tribunal unavailable — please retry shortly." },
			{ status: 502 },
		);
	}
	return NextResponse.json({ ok: run.ok, transcript: run.output });
}
