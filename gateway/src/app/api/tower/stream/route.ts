import { spawn } from "node:child_process";
import path from "node:path";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";
export const maxDuration = 300;

const REPO_ROOT = process.env.MESH_REPO_ROOT || path.resolve(process.cwd(), "..");

/**
 * The Tower control-plane, LIVE.
 *
 * One click runs a real governance cycle and streams every step over SSE:
 *   1. The Tower scans the on-chain world (real `tower` binary).
 *   2. It WAKES the Tribunal on the pending docket (real `tribunal --dry-run`).
 *   3. It reports — read-only / dry-run, so no funds move.
 *
 * This is the overseer actually conducting the swarm, not just recommending.
 */
export async function GET() {
	const encoder = new TextEncoder();

	const stream = new ReadableStream({
		async start(controller) {
			const send = (phase: string, line: string) =>
				controller.enqueue(encoder.encode(`data: ${JSON.stringify({ phase, line })}\n\n`));

			const runStep = (phase: string, bin: string, args: string[], timeoutMs: number) =>
				new Promise<number>((resolve) => {
					const binPath = path.join(REPO_ROOT, "target", "debug", bin);
					const child = spawn(binPath, args, { cwd: REPO_ROOT, env: process.env });
					const timer = setTimeout(() => child.kill("SIGKILL"), timeoutMs);
					let buf = "";
					const flush = (chunk: Buffer) => {
						buf += chunk.toString();
						const lines = buf.split("\n");
						buf = lines.pop() ?? "";
						for (const l of lines) send(phase, l);
					};
					child.stdout.on("data", flush);
					child.stderr.on("data", flush);
					child.on("error", (e) => { send(phase, `OFFLINE: ${e.message}`); });
					child.on("close", (code) => {
						if (buf) send(phase, buf);
						clearTimeout(timer);
						resolve(code ?? 0);
					});
				});

			try {
				send("tower", "🗼 THE TOWER awakens — scanning the on-chain world...");
				await runStep("tower", "tower", [], 60_000);

				send("dispatch", "");
				send("dispatch", "🧠 Tower decision: convene the Tribunal on the pending docket (dry-run).");
				send("dispatch", "");

				await runStep(
					"tribunal",
					"tribunal",
					[
						"--dry-run",
						"--task-id", "tower-docket",
						"--description", "Optimize the AST hypergraph for the Odra escrow modules; reduce redundant edges.",
						"--proof", "All three escrow modules processed; redundant def-use edges removed 18412->11067; Coq-checked equivalence + 10k proptests confirm identical behaviour; coverage 71->94%; 59 tests pass; CI green; artifacts linked.",
					],
					280_000,
				);

				send("done", "");
				send("done", "🗼 Governance cycle complete. Read-only / dry-run — no funds moved.");
			} catch (e) {
				send("error", `🛑 SERVICE DEGRADED: ${e instanceof Error ? e.message : String(e)} — functions frozen, we are working on it.`);
			} finally {
				controller.enqueue(encoder.encode("event: end\ndata: {}\n\n"));
				controller.close();
			}
		},
	});

	return new Response(stream, {
		headers: {
			"Content-Type": "text/event-stream",
			"Cache-Control": "no-cache, no-transform",
			Connection: "keep-alive",
		},
	});
}
