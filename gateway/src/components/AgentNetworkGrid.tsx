"use client";
import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import type { AgentRecord } from "@/lib/agent_registry";
import { HoverReel } from "@/components/HoverReel";

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

// Bespoke art per agent (user-generated, /public/cards). Cards without an entry keep the
// gradient look — the map is additive, zero risk to unknown/api-fed agents.
const AGENT_ART: Record<string, string> = {
	malicious_node_x9: "/cards/agent-malicious.webp",
	mark_53_sarcophagus: "/cards/agent-sarcophagus.webp",
	"x402-AEGIS-NODE": "/cards/agent-aegis.webp",
	casper_scrapper_v2: "/cards/agent-scrapper.webp",
	cortex_reviewer: "/cards/agent-cortex.webp",
	credio_risk_monitor: "/cards/agent-credio.webp",
	liquidity_sniper: "/cards/agent-sniper.webp",
	agent_alpha_arbitrage: "/cards/agent-alpha.webp",
};

const AGENT_REEL: Record<string, string> = {
	malicious_node_x9: "reel-malicious",
	mark_53_sarcophagus: "reel-sarcophagus",
};

function ProduxCard({ agent, spanClass }: { agent: AgentDisplay; spanClass: string }) {
	return (
		<motion.div
			className={`${spanClass} flex flex-col group cursor-pointer`}
			initial={{ opacity: 0, y: 50 }}
			whileInView={{ opacity: 1, y: 0 }}
			viewport={{ once: true, margin: "-10%" }}
			transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
		>
			<div className="focus-card editorial-panel relative overflow-hidden bg-[#0a0a0a] border border-white/5 w-full aspect-[4/3] max-sm:aspect-[4/5]">
				<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
				<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 30% 30%, var(--red-500) 0%, transparent 70%)" }} />
				{AGENT_ART[agent.id] && (
					<div
						className="absolute inset-0 bg-cover bg-center opacity-[0.82] transition-[transform,opacity] duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] group-hover:opacity-100 group-hover:scale-105"
						style={{ backgroundImage: `url(${AGENT_ART[agent.id]})` }}
					/>
				)}
				{AGENT_REEL[agent.id] && <HoverReel name={AGENT_REEL[agent.id]} />}
				<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay" />
				
				<div className="absolute bottom-6 right-6 flex gap-2 z-10 max-sm:bottom-4 max-sm:right-4">
					<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">
						{agent.status}
					</span>
					<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">
						REP: {agent.rep}
					</span>
				</div>
			</div>

			<div className="mt-[3vh] project-info-trigger relative flex w-full">
				<div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 mt-1 transition-colors group-hover:bg-white/20"></div>
				<div className="flex flex-col gap-[0.73vw]">
					<h3 className="heading-32 text-[var(--gray-1000)] m-0">{agent.id}</h3>
					<p className="label-13-mono text-[var(--gray-700)] uppercase max-w-[85%] leading-relaxed max-sm:max-w-full">
						{agent.task}. SECURING {agent.staked} ON CASPER NETWORK. PASSPORT: {agent.passport}.
					</p>
				</div>
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
					{AGENT_ART[agent.id] && (
						<div
							className="absolute inset-0 bg-cover bg-center opacity-[0.8] transition-[transform,opacity] duration-[1.5s] ease-[cubic-bezier(0.16,1,0.3,1)] group-hover:opacity-100 group-hover:scale-110"
							style={{ backgroundImage: `url(${AGENT_ART[agent.id]})` }}
						/>
					)}
					{AGENT_REEL[agent.id] && <HoverReel name={AGENT_REEL[agent.id]} />}
					<div className="absolute inset-0 group-hover:scale-110 transition-transform duration-[1.5s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay" />
					
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

			<div className="w-full grid grid-cols-12 gap-x-[4vw] gap-y-[15vh] px-[2vw] max-sm:gap-y-[10vh] mb-[15vh]">
				{agents.length > 0 && (
					<ProduxCard agent={agents[agents.length > 1 ? 1 : 0]} spanClass="col-span-12 md:col-span-6" />
				)}
				{agents.length > 1 && (
					<ProduxCard agent={agents[0]} spanClass="col-span-12 md:col-span-6" />
				)}
			</div>

			{agents.length > 2 && (
				<AccordionSection agents={agents.slice(2, 7)} />
			)}
		</div>
	);
}
