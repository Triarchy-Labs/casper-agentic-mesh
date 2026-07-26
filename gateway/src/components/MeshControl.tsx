"use client";

import React, { useState } from "react";
import Image from "next/image";
import { CornerMarks } from "@/components/AgentNetworkGrid";

// Colour a transcript line by the agent role that produced it.
function lineClass(line: string): string {
	if (line.includes("🗡️") || line.includes("PROSECUTION")) return "text-[var(--gray-1000)]";
	if (line.includes("🛡️") || line.includes("DEFENSE")) return "text-[var(--gray-700)]";
	if (line.includes("👨‍⚖️") || line.includes("RULING") || line.includes("CHIEF")) return "text-white font-semibold";
	if (line.includes("✅") || line.includes("APPROVE") || line.includes("ALIVE") || line.includes("healthy")) return "text-white";
	if (line.includes("❌") || line.includes("REJECT") || line.includes("🚨") || line.includes("STALE")) return "text-[var(--gray-1000)]";
	if (line.includes("🛑") || line.includes("DEGRADED") || line.includes("frozen")) return "text-[var(--gray-1000)] font-semibold";
	if (line.includes("⚠️") || line.includes("PARTIAL")) return "text-[var(--red-900)]";
	if (line.includes("🔗") || line.includes("https://")) return "text-[var(--red-900)] underline";
	if (line.startsWith("⚖️") || line.includes("JUROR")) return "text-white/70";
	return "text-white/50";
}

function Console({ lines }: { lines: string[] }) {
	return (
		<div
			data-lenis-prevent
			className="mt-4 max-h-[420px] overflow-y-auto overscroll-contain bg-black/60 border border-[var(--gray-500)] p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap"
			style={{ scrollbarWidth: "thin", scrollbarColor: "var(--red-700) rgba(255,255,255,0.08)", scrollbarGutter: "stable" }}
		>
			{lines.map((l, i) => (
				<div key={i} className={lineClass(l)}>{l || " "}</div>
			))}
		</div>
	);
}

export function MeshControl() {
	const [towerLines, setTowerLines] = useState<string[] | null>(null);
	const [towerBusy, setTowerBusy] = useState(false);

	const [desc, setDesc] = useState("Optimize the AST hypergraph for the Odra escrow modules; reduce redundant edges.");
	const [proof, setProof] = useState("PR #1 merged (commit 4ad2744). All three escrow modules processed; redundant def-use edges removed 18412->11067; Coq-checked equivalence + 10k proptests confirm identical behaviour; coverage 71->94%; 59 tests pass; CI green; artifacts linked.");
	const [arena, setArena] = useState<string[] | null>(null);
	const [arenaBusy, setArenaBusy] = useState(false);
	const [judgeKey, setJudgeKey] = useState("");
	const [txBusy, setTxBusy] = useState(false);

	const fireLiveTx = async () => {
		setTxBusy(true); setTowerLines(["⛓️  signing a real register_agent TransactionV1 and submitting to testnet… (~20–60s)"]);
		try {
			const r = await fetch("/api/demo-tx", { method: "POST" });
			const d = await r.json();
			setTowerLines(d.lines?.length ? d.lines : [d.error ?? "Live-tx demo unavailable."]);
		} catch {
			setTowerLines(["🛑 SERVICE DEGRADED: could not reach the signer host. No fabricated hashes — we are working on it."]);
		} finally { setTxBusy(false); }
	};

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
				body: JSON.stringify({ description: desc, proof, ...(judgeKey.trim() ? { apiKey: judgeKey.trim() } : {}) }),
			});
			const d = await r.json();
			setArena(d.transcript ? d.transcript.split("\n") : [d.error ?? "Tribunal unavailable."]);
		} catch {
			setArena(["🛑 SERVICE DEGRADED: the tribunal is unreachable. No funds moved — we are working on it."]);
		} finally { setArenaBusy(false); }
	};

	return (
		<div className="col-span-12 flex flex-col md:flex-row gap-[12%] items-end mb-[10vh]">
			{/* THE TOWER */}
			<div className="w-full md:w-[58%] flex flex-col">
				<div className="focus-card editorial-panel block relative w-full aspect-[5/4] max-sm:aspect-auto max-sm:min-h-[50vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
					{/* Produx Visual Background */}
					<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
					<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 30% 30%, var(--red-500) 0%, transparent 70%)" }} />
					{/* the overseer's control room — near-full brightness (5% veil at rest, none on hover) */}
					<div className="card-art absolute inset-0 bg-black opacity-[0.95] group-hover:opacity-100 transition-opacity duration-700">
                        <Image src="/cards/tower-control.webp" alt="Tower Control" fill style={{ objectFit: "cover" }} quality={95} priority={false} />
                    </div>
					<div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent" />
					<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
					
					<div className="absolute inset-0 p-[2.22vw] pb-[80px] z-10 flex flex-col justify-center overflow-y-auto">
						<CornerMarks />
						<div className="flex items-center justify-between mb-2">
							<span className="text-[10px] tracking-widest lowercase text-white/30">read-only</span>
						</div>
						<p className="text-xs text-white/40 lowercase leading-relaxed mb-5">
							one brain over the swarm. reads the on-chain world — oracle, reputation,
							agent liveness — then dispatches sub-agents. ◆ click to scan. nothing runs
							in the background. nothing is spent.
						</p>
						{towerLines && <Console lines={towerLines} />}
					</div>
					<button
						onClick={scanTower}
						disabled={towerBusy}
						className="font-DM-mono border border-white/5 bg-black/25 px-[2.5vw] py-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:px-[3.5vw] max-lg:py-[0.97vw] max-sm:text-[9px] max-sm:px-4 max-sm:py-2 rounded-none absolute bottom-6 left-6 z-20 transition-all hover:bg-white/10 hover:border-[var(--red-700)] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{towerBusy ? "scanning..." : "scan mesh"}
					</button>
					<div className="absolute bottom-6 right-6 flex gap-2 z-20">
						<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Overseer</span>
						<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Liveness</span>
					</div>
				</div>
				<div className="mt-[4vh] project-info-trigger relative flex w-full items-start justify-between gap-4">
					<div className="flex">
						<div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 mt-1 transition-colors group-hover:bg-white/20"></div>
						<div className="flex flex-col gap-[0.73vw]">
							<h3 className="heading-32 leading-tight">The Tower</h3>
							<p className="label-13-mono text-[var(--gray-800)] uppercase">Mesh Overseer. Analyzes chain state.</p>
						</div>
					</div>
					<button
						onClick={fireLiveTx}
						disabled={txBusy || towerBusy}
						className="font-DM-mono shrink-0 border border-[var(--red-700)]/40 bg-black/25 px-[2.5vw] py-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:px-[3.5vw] max-lg:py-[0.97vw] max-sm:text-[9px] max-sm:px-4 max-sm:py-2 rounded-none transition-all hover:bg-[var(--red-700)]/20 hover:border-[var(--red-700)] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{txBusy ? "signing on-chain..." : "fire live tx"}
					</button>
				</div>
			</div>

			{/* AGENT TRIBUNAL / ARENA */}
			<div className="w-full md:w-[30%] flex flex-col">
				<div className="focus-card editorial-panel block relative w-full aspect-square max-sm:aspect-auto max-sm:min-h-[60vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
					{/* Produx Visual Background */}
					<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
					<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 70% 70%, var(--red-500) 0%, transparent 70%)" }} />
					{/* the arena: the empty crimson rooftop — the stage the court convenes over */}
					<div className="card-art absolute inset-0 bg-black opacity-[0.45] group-hover:opacity-[0.58] transition-opacity duration-700">
                        <Image src="/cards/tribunal-arena.webp" alt="Tribunal Arena" fill style={{ objectFit: "cover" }} quality={95} priority={false} />
                    </div>
					<div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-black/25" />
					<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
					
					<div data-lenis-prevent className="absolute inset-0 p-[2.22vw] pb-[80px] z-10 flex flex-col overflow-y-auto overflow-x-hidden overscroll-contain" style={{ scrollbarWidth: "none" }}>
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
							<input
								value={judgeKey} onChange={(e) => setJudgeKey(e.target.value)}
								type="password" autoComplete="off" spellCheck={false}
								placeholder="own OpenRouter key (optional — used once, never stored)"
								className="bg-black/60 border border-white/10 p-3 font-mono text-xs text-white/60 focus:border-[var(--red-700)] outline-none"
							/>
						</div>
						{arenaBusy && !arena && (
							<div className="mt-4 text-xs text-white/40 font-mono animate-pulse">
								the bench is deliberating across multiple models — this takes ~30–60s…
							</div>
						)}
						{arena && <Console lines={arena} />}
					</div>
					<button
						onClick={convene}
						disabled={arenaBusy}
						className="font-DM-mono border border-white/5 bg-black/25 px-[2.5vw] py-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:px-[3.5vw] max-lg:py-[0.97vw] max-sm:text-[9px] max-sm:px-4 max-sm:py-2 rounded-none absolute bottom-6 left-6 z-20 transition-all hover:bg-white/10 hover:border-[var(--red-700)] active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{arenaBusy ? "deliberating..." : "convene tribunal"}
					</button>
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
