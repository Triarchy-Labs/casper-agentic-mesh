"use client";
import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import type { AgentRecord } from "@/lib/agent_registry";

// Fallback data if /api/agents is unreachable
// Fallback data if /api/agents is unreachable
const FALLBACK_AGENTS = [
	{ id: "x402-AEGIS-NODE", task: "Security Matrix", rep: "99.9", earned: "1,240.50", status: "ACTIVE", staked: "500,000 CSPR", passport: "CEP78-SB-0042" },
	{ id: "agent_alpha_arbitrage", task: "DEX Arbitrage", rep: "95.0", earned: "420.00", status: "ACTIVE", staked: "150,000 CSPR", passport: "CEP78-SB-8902" },
	{ id: "casper_scrapper_v2", task: "Data Injection", rep: "88.5", earned: "110.20", status: "IDLE", staked: "45,000 CSPR", passport: "CEP78-SB-7719" },
	{ id: "malicious_node_x9", task: "Phishing Attempt", rep: "12.0", earned: "0.00", status: "QUARANTINED", staked: "0 CSPR", passport: "CEP78-SB-666D" },
	{ id: "cortex_reviewer", task: "Code Audit", rep: "97.2", earned: "890.00", status: "ACTIVE", staked: "180,000 CSPR", passport: "CEP78-SB-2901" },
	{ id: "liquidity_sniper", task: "Flash Loans", rep: "91.4", earned: "3,400.10", status: "ACTIVE", staked: "350,000 CSPR", passport: "CEP78-SB-5092" },
	{ id: "mark_53_sarcophagus", task: "Casper Autonomous Engine", rep: "100.0", earned: "Reference Protocol", status: "GOLDEN_TEMPLATE", staked: "1,000,000 CSPR", passport: "CEP78-SB-9999" },
];

interface AgentDisplay {
	id: string;
	task: string;
	rep: string;
	earned: string;
	status: string;
	staked: string;
	passport: string;
}

export function CornerMarks() {
	return (
		<>
			<span className="corner-mark corner-mark-tl">+</span>
			<span className="corner-mark corner-mark-tr">+</span>
			<span className="corner-mark corner-mark-bl">+</span>
			<span className="corner-mark corner-mark-br">+</span>
		</>
	);
}

function ProduxCard({ agent, spanClass }: { agent: AgentDisplay; spanClass: string }) {
	return (
		<motion.div
			className={`${spanClass} flex flex-col gap-[1.5vw] group cursor-pointer`}
			initial={{ opacity: 0, y: 50 }}
			whileInView={{ opacity: 1, y: 0 }}
			viewport={{ once: true }}
			transition={{ duration: 0.8 }}
		>
			<div className="w-full relative overflow-hidden bg-[#0a0a0a] aspect-[4/3] border border-white/5">
				<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
				<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 30% 30%, var(--red-500) 0%, transparent 70%)" }} />
				<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 mix-blend-overlay" />
				
				<div className="absolute bottom-6 right-6 flex gap-2 z-10 max-sm:bottom-4 max-sm:right-4">
					<span className="px-3 py-1 bg-black/40 backdrop-blur-md border border-white/10 label-10-mono text-white/70 uppercase">
						{agent.status}
					</span>
					<span className="px-3 py-1 bg-black/40 backdrop-blur-md border border-white/10 label-10-mono text-white/70 uppercase">
						REP: {agent.rep}
					</span>
				</div>
			</div>

			<div className="flex flex-col gap-[1vw] mt-[0.5vw]">
				<div className="flex items-center gap-[0.7vw]">
					<div className="size-[0.4vw] border border-[var(--gray-600)] shrink-0" />
					<h3 className="heading-24 text-[var(--gray-1000)] m-0">{agent.id}</h3>
				</div>
				<p className="label-13-mono text-[var(--gray-700)] uppercase max-w-[85%] leading-relaxed max-sm:max-w-full">
					{agent.task}. SECURING {agent.staked} ON CASPER NETWORK. PASSPORT: {agent.passport}.
				</p>
			</div>
		</motion.div>
	);
}

function AccordionSection({ agents }: { agents: AgentDisplay[] }) {
	return (
		<div className="w-full h-[60vh] flex gap-2 mt-[10vh] max-sm:flex-col max-sm:h-[120vh]">
			{agents.map((agent, i) => (
				<div 
					key={agent.id}
					className="group relative flex-1 hover:flex-[3] max-sm:hover:flex-[2] transition-all duration-[1s] ease-[cubic-bezier(0.16,1,0.3,1)] overflow-hidden cursor-pointer bg-[#0a0a0a] border border-white/5"
				>
					<div className="absolute inset-0 bg-gradient-to-t from-[#111] to-[#050505]" />
					{/* Unique subtle tint per card */}
					<div className="absolute inset-0 opacity-[0.15]" style={{ background: `radial-gradient(circle at 50% 100%, hsl(${i * 60}, 60%, 40%) 0%, transparent 70%)` }} />
					<div className="absolute inset-0 group-hover:scale-110 transition-transform duration-[1.5s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 mix-blend-overlay" />
					
					{/* Overlay Gradient for Text */}
					<div className="absolute bottom-0 left-0 right-0 p-6 flex flex-col justify-end bg-gradient-to-t from-black/90 via-black/40 to-transparent h-2/3 pointer-events-none" />

					{/* Title Content */}
					<div className="absolute bottom-6 left-6 right-6 h-[80px]">
						{/* Active State (Horizontal) */}
						<div className="absolute bottom-0 left-0 w-full opacity-0 group-hover:opacity-100 group-hover:translate-y-0 translate-y-4 transition-all duration-700 delay-100 ease-[cubic-bezier(0.16,1,0.3,1)]">
							<h3 className="heading-24 text-white m-0 truncate">{agent.id}</h3>
							<p className="label-12-mono text-white/50 mt-2 truncate">
								{agent.task} // {agent.status}
							</p>
						</div>
						
						{/* Inactive State (Vertical) */}
						<div className="absolute bottom-0 left-0 origin-bottom-left -rotate-90 opacity-100 group-hover:opacity-0 transition-opacity duration-300 whitespace-nowrap max-sm:hidden">
							<span className="heading-24 text-white/40 tracking-widest uppercase">{agent.id}</span>
						</div>
					</div>
				</div>
			))}
		</div>
	);
}

export default function AgentNetworkGrid() {
	const [agents, setAgents] = useState<AgentDisplay[]>(FALLBACK_AGENTS);

	useEffect(() => {
		const fetchAgents = async () => {
			try {
				const res = await fetch("/api/agents");
				if (res.ok) {
					const data = await res.json();
					if (data.agents && data.agents.length > 0) {
						const mapped: AgentDisplay[] = data.agents.map((a: AgentRecord) => ({
							id: a.id,
							task: a.capabilities?.[0] || "General",
							rep: a.reputationScore?.toFixed(1) || "0.0",
							earned: `${(a.csprSettled || 0).toFixed(2)}`,
							status: a.status?.toUpperCase() || "IDLE",
							staked: `${(a.stakedCollateralCspr || 0).toLocaleString()} CSPR`,
							passport: a.passportId || "CEP78-SB-XXXX"
						}));
						// Always append Mark 53 golden template
						const hasMark53 = mapped.some(a => a.id === "mark_53_sarcophagus");
						if (!hasMark53) {
							mapped.push(FALLBACK_AGENTS.find(a => a.id === "mark_53_sarcophagus") || FALLBACK_AGENTS[6]);
						}
						
						// Refill to 7 nodes so visual masonry grid is preserved
						for (const fb of FALLBACK_AGENTS) {
							if (mapped.length >= 7) break;
							if (!mapped.find(a => a.id === fb.id)) {
								mapped.push(fb);
							}
						}
						setAgents(mapped);
					}
				}
			} catch {
				// Keep fallback data
			}
		};
		fetchAgents();
		const interval = setInterval(fetchAgents, 5000);
		return () => clearInterval(interval);
	}, []);

	return (
		<div
			id="agent-network"
			className="w-full"
			style={{
				paddingTop: "40px",
				paddingBottom: "40px",
				background: "transparent",
				color: "var(--gray-1000)",
				fontFamily: "var(--font-mono)",
			}}
		>
			<div style={{
				width: "100%",
				margin: "0 auto 40px",
				height: "1px",
				background: `linear-gradient(90deg, transparent, var(--gray-400), transparent)`,
			}} />

			<motion.div
				initial={{ opacity: 0, y: 10 }}
				whileInView={{ opacity: 1, y: 0 }}
				viewport={{ once: true }}
				transition={{ duration: 0.8 }}
				style={{
					display: "flex",
					justifyContent: "space-between",
					alignItems: "center",
					marginBottom: "24px",
				}}
			>
				<h2 className="label-14-mono text-[var(--gray-700)] m-0">
					{"// LIVE AGENT REGISTRY"}
				</h2>
				<div className="label-14-mono text-[var(--gray-700)]">
					NODES: {agents.length}
				</div>
			</motion.div>

			<div className="grid grid-cols-1 md:grid-cols-12 gap-x-[2vw] gap-y-[10vh] mb-[10vh]">
				{agents.length > 0 && (
					<ProduxCard agent={agents[0]} spanClass="col-span-12 md:col-span-7" />
				)}
				{agents.length > 1 && (
					<ProduxCard agent={agents[1]} spanClass="col-span-12 md:col-span-4 md:col-end-13 md:mt-[10vh]" />
				)}
			</div>

			{agents.length > 2 && (
				<AccordionSection agents={agents.slice(2, 7)} />
			)}
		</div>
	);
}
