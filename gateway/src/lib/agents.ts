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
	extraEnv?: Record<string, string>,
): Promise<AgentRun> {
	const binPath = path.join(REPO_ROOT, "target", "debug", bin);
	return new Promise((resolve) => {
		execFile(
			binPath,
			args,
			{ cwd: REPO_ROOT, timeout: timeoutMs, env: { ...process.env, ...extraEnv }, maxBuffer: 1024 * 1024 },
			(err, stdout, stderr) => {
				const output = `${stdout || ""}${stderr ? `\n${stderr}` : ""}`.trim();
				if (err && !output) {
					// On serverless the compiled agents simply aren't there. Say so in plain words —
					// a raw ENOENT spawn trace is noise to anyone reading the panel.
					const missing = (err as NodeJS.ErrnoException).code === "ENOENT";
					resolve({
						ok: false,
						output: "",
						error: missing
							? "🛑 FUNCTIONS FROZEN: this agent is a compiled Rust binary and cannot run on serverless. No funds moved, nothing faked — run it yourself in one command, see PLAYBOOK.md."
							: err.message,
					});
				} else {
					resolve({ ok: !err, output });
				}
			},
		);
	});
}
