"use client";

import React, { useState } from "react";
import { CornerMarks } from "@/components/AgentNetworkGrid";

// Colour a transcript line by the agent role that produced it.
function lineClass(line: string): string {
	if (line.includes("🗡️") || line.includes("PROSECUTION")) return "text-[#f13242]";
	if (line.includes("🛡️") || line.includes("DEFENSE")) return "text-sky-400";
	if (line.includes("👨‍⚖️") || line.includes("RULING") || line.includes("CHIEF")) return "text-amber-300";
	if (line.includes("✅") || line.includes("APPROVE") || line.includes("ALIVE") || line.includes("healthy")) return "text-white";
	if (line.includes("❌") || line.includes("REJECT") || line.includes("🚨") || line.includes("STALE")) return "text-[#f13242]";
	if (line.includes("🛑") || line.includes("DEGRADED") || line.includes("frozen")) return "text-[#f13242] font-semibold";
	if (line.includes("⚠️") || line.includes("PARTIAL")) return "text-amber-300";
	if (line.includes("🔗") || line.includes("https://")) return "text-[var(--red-900)] underline";
	if (line.startsWith("⚖️") || line.includes("JUROR")) return "text-white/70";
	return "text-white/50";
}

function Console({ lines }: { lines: string[] }) {
	return (
		<div className="mt-4 max-h-[420px] overflow-auto rounded-sm bg-black/60 border border-white/10 p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap">
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
			className="px-5 py-2.5 text-xs tracking-widest uppercase font-mono border border-[#f13242]/60 text-[#f13242] hover:bg-[#f13242] hover:text-black transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
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
		<div className="col-span-12 grid grid-cols-12 gap-8">
			{/* THE TOWER */}
			<div className="col-span-12 lg:col-span-5 editorial-panel p-8 relative">
				<CornerMarks />
				<div className="flex items-center justify-between mb-2">
					<h3 className="text-sm tracking-[0.2em] uppercase text-white/80"><span className="text-[var(--red-700)]">▚</span> The Tower · Overseer</h3>
					<span className="text-[10px] tracking-widest uppercase text-white/30">read-only</span>
				</div>
				<p className="text-xs text-white/40 leading-relaxed mb-5">
					One brain over the swarm. Reads the on-chain world (oracle, reputation,
					agent liveness), then dispatches sub-agents. Click to scan — nothing runs
					in the background, nothing is spent.
				</p>
				<Btn onClick={scanTower} busy={towerBusy}>Scan mesh</Btn>
				{towerLines && <Console lines={towerLines} />}
			</div>

			{/* AGENT TRIBUNAL / ARENA */}
			<div className="col-span-12 lg:col-span-7 editorial-panel p-8 relative">
				<CornerMarks />
				<div className="flex items-center justify-between mb-2">
					<h3 className="text-sm tracking-[0.2em] uppercase text-white/80"><span className="text-[var(--red-700)]">▞</span> Agent Tribunal · Arena</h3>
					<span className="text-[10px] tracking-widest uppercase text-white/50">dry-run · no funds moved</span>
				</div>
				<p className="text-xs text-white/40 leading-relaxed mb-5">
					An adversarial court of real models — prosecutor, defender, a jury of
					diverse LLMs, and a chief judge — deliberates a bounty. On a real run the
					verdict moves CSPR on-chain; here it deliberates only.
				</p>
				<div className="grid gap-3 mb-4">
					<textarea
						value={desc} onChange={(e) => setDesc(e.target.value)} rows={2}
						placeholder="Task description"
						className="bg-black/60 border border-white/10 p-3 font-mono text-xs text-white/80 focus:border-[#f13242]/50 outline-none resize-none"
					/>
					<textarea
						value={proof} onChange={(e) => setProof(e.target.value)} rows={3}
						placeholder="Submitted proof"
						className="bg-black/60 border border-white/10 p-3 font-mono text-xs text-white/80 focus:border-[#f13242]/50 outline-none resize-none"
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
		</div>
	);
}
