import { execFile } from "node:child_process";
import path from "node:path";

// Repo root relative to the gateway working dir (Next runs with cwd=gateway).
const REPO_ROOT = process.env.MESH_REPO_ROOT || path.resolve(process.cwd(), "..");

export interface AgentRun {
	ok: boolean;
	output: string;
	error?: string;
}

/**
 * Spawn a compiled mesh agent binary and capture its output.
 * Agents invoked here are SAFE: the Tower is read-only and the Tribunal runs
 * with --dry-run (no signer, no funds moved).
 */
export function runAgent(
	bin: string,
	args: string[],
	timeoutMs: number,
): Promise<AgentRun> {
	const binPath = path.join(REPO_ROOT, "target", "debug", bin);
	return new Promise((resolve) => {
		execFile(
			binPath,
			args,
			{ cwd: REPO_ROOT, timeout: timeoutMs, env: process.env, maxBuffer: 1024 * 1024 },
			(err, stdout, stderr) => {
				const output = `${stdout || ""}${stderr ? `\n${stderr}` : ""}`.trim();
				if (err && !output) {
					resolve({ ok: false, output: "", error: err.message });
				} else {
					resolve({ ok: !err, output });
				}
			},
		);
	});
}
