import { NextResponse } from "next/server";
import { execFile } from "node:child_process";
import path from "node:path";
import fs from "node:fs";

export const dynamic = "force-dynamic";
export const maxDuration = 300;

const REPO_ROOT = process.env.MESH_REPO_ROOT || path.resolve(process.cwd(), "..");

// Fire the same live demo the PLAYBOOK runs in a terminal: a REAL register_agent
// TransactionV1 signed and submitted to Casper testnet. Only possible on a host that
// has the repo + go-signer binary (localhost / VM). On serverless it degrades honestly.
export async function POST() {
	const script = path.join(REPO_ROOT, "run_demo.sh");
	if (!fs.existsSync(script)) {
		return NextResponse.json(
			{ ok: false, error: "Live-tx demo offline: script not present on this host (serverless). Run ./run_demo.sh from the PLAYBOOK — same thing, same chain." },
			{ status: 503 },
		);
	}
	const run = await new Promise<{ ok: boolean; output: string }>((resolve) => {
		execFile("bash", [script], { cwd: REPO_ROOT, timeout: 240_000, env: process.env, maxBuffer: 1024 * 1024 }, (err, stdout, stderr) => {
			const output = `${stdout || ""}${stderr ? `\n${stderr}` : ""}`.trim();
			resolve({ ok: !err && !!output, output });
		});
	});

	// strip ANSI colors for the UI console
	// eslint-disable-next-line no-control-regex
	const clean = run.output.replace(/\[[0-9;]*m/g, "");
	const tx = clean.match(/https:\/\/testnet\.cspr\.live\/transaction\/[0-9a-f]{64}/)?.[0] ?? null;
	return NextResponse.json({
		ok: run.ok && !!tx,
		txUrl: tx,
		lines: clean.split("\n").filter(Boolean),
	});
}
