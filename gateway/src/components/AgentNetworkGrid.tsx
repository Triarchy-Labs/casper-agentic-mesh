"use client";
import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import type { AgentRecord } from "@/lib/agent_registry";

// Fallback data if /api/agents is unreachable
// Fallback data if /api/agents is unreachable
const FALLBACK_AGENTS = [
	{ id: "x402-AEGIS-NODE", task: "Security Matrix", rep: "99.9", earned: "$1,240.50", status: "ACTIVE", staked: "500,000 CSPR", passport: "CEP78-SB-0042" },
	{ id: "agent_alpha_arbitrage", task: "DEX Arbitrage", rep: "95.0", earned: "$420.00", status: "ACTIVE", staked: "150,000 CSPR", passport: "CEP78-SB-8902" },
	{ id: "casper_scrapper_v2", task: "Data Injection", rep: "88.5", earned: "$110.20", status: "IDLE", staked: "45,000 CSPR", passport: "CEP78-SB-7719" },
	{ id: "malicious_node_x9", task: "Phishing Attempt", rep: "12.0", earned: "$0.00", status: "QUARANTINED", staked: "0 CSPR", passport: "CEP78-SB-666D" },
	{ id: "cortex_reviewer", task: "Code Audit", rep: "97.2", earned: "$890.00", status: "ACTIVE", staked: "180,000 CSPR", passport: "CEP78-SB-2901" },
	{ id: "liquidity_sniper", task: "Flash Loans", rep: "91.4", earned: "$3,400.10", status: "ACTIVE", staked: "350,000 CSPR", passport: "CEP78-SB-5092" },
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

function AgentCard({ agent, index }: { agent: AgentDisplay; index: number }) {
	const [hovered, setHovered] = useState(false);
	const isQuarantined = agent.status === "QUARANTINED";
	const isMark53 = agent.id === "mark_53_sarcophagus";

	let statusColor = "var(--gray-700)";
	if (agent.status === "ACTIVE") statusColor = "var(--gray-1000)";
	else if (agent.status === "QUARANTINED") statusColor = "var(--red-700)";
	else if (agent.status === "GOLDEN_TEMPLATE") statusColor = "var(--red-700)";

	let borderColor = isQuarantined 
		? "var(--red-700)" 
		: hovered 
			? "var(--gray-600)"
			: "var(--gray-400)";

	if (isMark53) {
		borderColor = "transparent"; // Handled by rare-snake-border pseudo-element
	}

	return (
		<motion.div
			className={`editorial-panel bento-card bento-card-${index} ${isMark53 ? "rare-snake-border" : ""}`}
			initial={{ opacity: 0, y: 50, rotateX: 5, scale: 0.98 }}
			whileInView={{ opacity: 1, y: 0, rotateX: 0, scale: 1 }}
			whileHover={{ y: -8, scale: 1.015, zIndex: 10 }}
			viewport={{ once: true, amount: 0.1 }}
			transition={{ duration: 1.0, ease: [0.16, 1, 0.3, 1], delay: (index % 2) * 0.1 }}
			onMouseEnter={() => setHovered(true)}
			onMouseLeave={() => setHovered(false)}
			style={{
				transformStyle: "preserve-3d",
				transformOrigin: `top ${index % 2 === 0 ? "left" : "right"}`,
				padding: "32px",
				backgroundColor: hovered ? "var(--gray-200)" : "var(--background-100)",
				border: isMark53 ? "none" : `1px solid ${borderColor}`,
				borderRadius: "0px",
				display: "flex",
				flexDirection: "column",
				gap: "20px",
				cursor: "crosshair",
				transition: "background 0.35s ease, border-color 0.35s ease, box-shadow 0.35s ease",
				boxShadow: isMark53 
					? "0 0 40px rgba(241, 50, 66, 0.15)"
					: hovered 
						? "0 20px 40px rgba(0,0,0,0.5)" 
						: "none",
				minHeight: index === 0 || index === 3 || index === 6 ? "240px" : "320px",
				justifyContent: "space-between",
				overflow: "hidden",
				wordBreak: "break-word" as const,
				maxWidth: "100%",
				boxSizing: "border-box" as const,
				position: "relative"
			}}
		>
			<CornerMarks />
			
			<div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", zIndex: 6 }}>
				<span className="label-14-mono text-[var(--gray-1000)]" style={{ fontWeight: 600 }}>
					{agent.id}
				</span>
				<span
					className="label-12-mono"
					style={{
						padding: "4px 8px",
						border: `1px solid ${statusColor}`,
						color: statusColor,
						borderRadius: "4px",
						fontWeight: 600
					}}
				>
					{agent.status}
				</span>
			</div>

			<div style={{ display: "flex", flexDirection: "column", gap: "8px", borderTop: "1px solid var(--gray-400)", paddingTop: "12px", zIndex: 6 }}>
				<div style={{ display: "flex", justifyContent: "space-between" }} className="label-12-mono text-[var(--gray-700)]">
					<span>PASSPORT:</span>
					<span className="text-[var(--gray-1000)]">{agent.passport}</span>
				</div>
				<div style={{ display: "flex", justifyContent: "space-between" }} className="label-12-mono text-[var(--gray-700)]">
					<span>COLLATERAL:</span>
					<span className="text-[var(--gray-1000)]">{agent.staked}</span>
				</div>
			</div>
			
			<div 
				className="label-14-mono text-[var(--gray-800)]"
				style={{ 
					display: "flex", 
					justifyContent: "space-between", 
					alignItems: "flex-end",
					marginTop: "auto",
					zIndex: 6
				}}
			>
				<div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
					<span className="text-[10px] text-[var(--gray-600)]">DOMAIN</span>
					<span className="text-[var(--gray-1000)]">[{agent.task}]</span>
				</div>

				<div style={{ display: "flex", flexDirection: "column", gap: "4px", alignItems: "flex-end" }}>
					<div style={{ display: "flex", gap: "16px", alignItems: "center" }}>
						<span>REP: {agent.rep}</span>
						<span style={{ color: isMark53 ? statusColor : "var(--gray-1000)" }}>
							{isMark53 ? "" : "USDC: "}{agent.earned}
						</span>
					</div>
					{isMark53 && (
						<div style={{ display: "flex", gap: "8px", marginTop: "12px" }}>
							<button 
								onClick={() => window.open("https://github.com/Triarchy-Labs/mark53-autonomous-node", "_blank")}
								className="button-secondary label-12-mono"
								style={{
									padding: "4px 8px",
									height: "28px",
									fontSize: "9px",
									cursor: "pointer",
									opacity: hovered ? 1 : 0.7
								}}
							>
								[ OS CONFIG ]
							</button>
							<button 
								onClick={() => window.open("https://github.com/Triarchy-Labs/tauri-exosuit-gateway", "_blank")}
								className="button-primary label-12-mono"
								style={{
									padding: "4px 8px",
									height: "28px",
									fontSize: "9px",
									fontWeight: "bold",
									cursor: "pointer",
									backgroundColor: "var(--red-700)",
									color: "#fff",
									opacity: hovered ? 1 : 0.8
								}}
							>
								[ TAURI EXOSUIT ]
							</button>
						</div>
					)}
				</div>
			</div>
		</motion.div>
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
							earned: `$${(a.usdcSettled || 0).toFixed(2)}`,
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
			style={{
				padding: "40px 0",
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

			<div className="bento-grid">
				{agents.map((agent: AgentDisplay, i: number) => (
					<AgentCard key={agent.id} agent={agent} index={i} />
				))}
			</div>
		</div>
	);
}
