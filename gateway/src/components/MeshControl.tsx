"use client";

import React, { useState } from "react";
import { CornerMarks } from "@/components/AgentNetworkGrid";

// Colour a transcript line by the agent role that produced it.
function lineClass(line: string): string {
	if (line.includes("🗡️") || line.includes("PROSECUTION")) return "text-[var(--gray-1000)]";
	if (line.includes("🛡️") || line.includes("DEFENSE")) return "text-[var(--gray-700)]";
	if (line.includes("👨‍⚖️") || line.includes("RULING") || line.includes("CHIEF")) return "text-amber-300";
	if (line.includes("✅") || line.includes("APPROVE") || line.includes("ALIVE") || line.includes("healthy")) return "text-white";
	if (line.includes("❌") || line.includes("REJECT") || line.includes("🚨") || line.includes("STALE")) return "text-[var(--gray-1000)]";
	if (line.includes("🛑") || line.includes("DEGRADED") || line.includes("frozen")) return "text-[var(--gray-1000)] font-semibold";
	if (line.includes("⚠️") || line.includes("PARTIAL")) return "text-amber-300";
	if (line.includes("🔗") || line.includes("https://")) return "text-[var(--red-900)] underline";
	if (line.startsWith("⚖️") || line.includes("JUROR")) return "text-white/70";
	return "text-white/50";
}

function Console({ lines }: { lines: string[] }) {
	return (
		<div className="mt-4 max-h-[420px] overflow-auto bg-black/60 border border-[var(--gray-500)] p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap">
			{lines.map((l, i) => (
				<div key={i} className={lineClass(l)}>{l || " "}</div>
			))}
		</div>
	);
}

function Btn({ onClick, busy, children }: { onClick: () => void; busy: boolean; children: React.ReactNode }) {
	return (
		<button
			onClick={onClick}
			disabled={busy}
			className="button-secondary"
		>
			{busy ? "running…" : children}
		</button>
	);
}

export function MeshControl() {
	const [towerLines, setTowerLines] = useState<string[] | null>(null);
	const [towerBusy, setTowerBusy] = useState(false);

	const [desc, setDesc] = useState("Optimize the AST hypergraph for the Odra escrow modules; reduce redundant edges.");
	const [proof, setProof] = useState("PR #1 merged (commit 4ad2744). All three escrow modules processed; redundant def-use edges removed 18412->11067; Coq-checked equivalence + 10k proptests confirm identical behaviour; coverage 71->94%; 59 tests pass; CI green; artifacts linked.");
	const [arena, setArena] = useState<string[] | null>(null);
	const [arenaBusy, setArenaBusy] = useState(false);

	const scanTower = async () => {
		setTowerBusy(true); setTowerLines(null);
		try {
			const r = await fetch("/api/tower", { method: "POST" });
			const d = await r.json();
			setTowerLines(d.lines ?? [d.error ?? "Tower unavailable."]);
		} catch {
			setTowerLines(["🛑 SERVICE DEGRADED: could not reach the Tower. Functions are frozen — we are working on it."]);
		} finally { setTowerBusy(false); }
	};

	const convene = async () => {
		setArenaBusy(true); setArena(null);
		try {
			const r = await fetch("/api/tribunal", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ description: desc, proof }),
			});
			const d = await r.json();
			setArena(d.transcript ? d.transcript.split("\n") : [d.error ?? "Tribunal unavailable."]);
		} catch {
			setArena(["🛑 SERVICE DEGRADED: the tribunal is unreachable. No funds moved — we are working on it."]);
		} finally { setArenaBusy(false); }
	};

	return (
		<div className="col-span-12 flex flex-col md:flex-row gap-[12%] items-end focus-cards mb-[10vh]">
			{/* THE TOWER */}
			<div className="w-full md:w-[58%] flex flex-col">
				<div className="editorial-panel block relative w-full aspect-[5/4] max-sm:aspect-auto max-sm:min-h-[50vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
					{/* Produx Visual Background */}
					<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
					<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 30% 30%, var(--red-500) 0%, transparent 70%)" }} />
					<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
					
					<div className="absolute inset-0 p-[2.22vw] z-10 flex flex-col justify-center overflow-y-auto">
						<CornerMarks />
						<div className="flex items-center justify-between mb-2">
							<span className="text-[10px] tracking-widest lowercase text-white/30">read-only</span>
						</div>
						<p className="text-xs text-white/40 lowercase leading-relaxed mb-5">
							one brain over the swarm. reads the on-chain world — oracle, reputation,
							agent liveness — then dispatches sub-agents. ◆ click to scan. nothing runs
							in the background. nothing is spent.
						</p>
						<Btn onClick={scanTower} busy={towerBusy}>Scan mesh</Btn>
						{towerLines && <Console lines={towerLines} />}
					</div>
					<div className="absolute bottom-6 right-6 flex gap-2 z-20">
						<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Overseer</span>
						<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Liveness</span>
					</div>
				</div>
				<div className="mt-[4vh] project-info-trigger relative flex w-full">
					<div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 mt-1 transition-colors group-hover:bg-white/20"></div>
					<div className="flex flex-col gap-[0.73vw]">
						<h3 className="heading-32 leading-tight">The Tower</h3>
						<p className="label-13-mono text-[var(--gray-800)] uppercase">Mesh Overseer. Analyzes chain state.</p>
					</div>
				</div>
			</div>

			{/* AGENT TRIBUNAL / ARENA */}
			<div className="w-full md:w-[30%] flex flex-col">
				<div className="editorial-panel block relative w-full aspect-square max-sm:aspect-auto max-sm:min-h-[60vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
					{/* Produx Visual Background */}
					<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
					<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 70% 70%, var(--red-500) 0%, transparent 70%)" }} />
					<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
					
					<div className="absolute inset-0 p-[2.22vw] z-10 flex flex-col overflow-y-auto">
						<CornerMarks />
						<div className="flex items-center justify-between mb-2">
							<span className="text-[10px] tracking-widest lowercase text-white/50">dry-run · no funds moved</span>
						</div>
						<p className="text-xs text-white/40 lowercase leading-relaxed mb-5">
							an adversarial court of real models — prosecutor, defender, a jury of
							diverse llms, and a chief judge — deliberates a bounty. on a real run the
							verdict moves cspr on-chain. ◆ here it deliberates only.
						</p>
						<div className="grid gap-3 mb-4">
							<textarea
								value={desc} onChange={(e) => setDesc(e.target.value)} rows={2}
								placeholder="Task description"
								className="bg-black/60 border border-white/10 p-3 font-mono text-xs text-white/80 focus:border-[var(--gray-1000)] outline-none resize-none"
							/>
							<textarea
								value={proof} onChange={(e) => setProof(e.target.value)} rows={3}
								placeholder="Submitted proof"
								className="bg-black/60 border border-white/10 p-3 font-mono text-xs text-white/80 focus:border-[var(--gray-1000)] outline-none resize-none"
							/>
						</div>
						<Btn onClick={convene} busy={arenaBusy}>Convene tribunal</Btn>
						{arenaBusy && !arena && (
							<div className="mt-4 text-xs text-white/40 font-mono animate-pulse">
								the bench is deliberating across multiple models — this takes ~30–60s…
							</div>
						)}
						{arena && <Console lines={arena} />}
					</div>
					<div className="absolute bottom-6 right-6 flex gap-2 z-20">
						<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Tribunal</span>
						<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Dry Run</span>
					</div>
				</div>
				<div className="mt-[4vh] project-info-trigger relative flex w-full">
					<div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 mt-1 transition-colors group-hover:bg-white/20"></div>
					<div className="flex flex-col gap-[0.73vw]">
						<h3 className="heading-32 leading-tight">Agent Tribunal</h3>
						<p className="label-13-mono text-[var(--gray-800)] uppercase">Adversarial arena. Jury resolves bounties.</p>
					</div>
				</div>
			</div>
		</div>
	);
}
