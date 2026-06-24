"use client";
import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";

// Fallback data if /api/agents is unreachable
const FALLBACK_AGENTS = [
	{ id: "x402-AEGIS-NODE", task: "Security Matrix", rep: "99.9", earned: "$1,240.50", status: "ACTIVE" },
	{ id: "agent_alpha_arbitrage", task: "DEX Arbitrage", rep: "95.0", earned: "$420.00", status: "ACTIVE" },
	{ id: "stellar_scrapper_v2", task: "Data Injection", rep: "88.5", earned: "$110.20", status: "IDLE" },
	{ id: "malicious_node_x9", task: "Phishing Attempt", rep: "12.0", earned: "$0.00", status: "QUARANTINED" },
	{ id: "cortex_reviewer", task: "Code Audit", rep: "97.2", earned: "$890.00", status: "ACTIVE" },
	{ id: "liquidity_sniper", task: "Flash Loans", rep: "91.4", earned: "$3,400.10", status: "ACTIVE" },
	{ id: "mark_53_sarcophagus", task: "Stellar Autonomous Engine", rep: "100.0", earned: "Reference Protocol", status: "GOLDEN_TEMPLATE" },
];

interface AgentDisplay {
	id: string;
	task: string;
	rep: string;
	earned: string;
	status: string;
}

const FONT_HEADING = "'Helvetica Now Display', 'Inter', sans-serif";
const lusionTransition = "all 0.35s cubic-bezier(0.4, 0, 0.2, 1)";

function AgentCard({ agent, theme, index }: { agent: AgentDisplay; theme: "dark" | "light"; index: number }) {
	const [hovered, setHovered] = useState(false);
	const isQuarantined = agent.status === "QUARANTINED";
	
	const isMark53 = agent.id === "mark_53_sarcophagus";
	
	let borderColor = isQuarantined 
		? "#ff003c" 
		: hovered 
			? (theme === "dark" ? "rgba(0,255,65,0.5)" : "rgba(0,100,34,0.4)")
			: (theme === "dark" ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.1)");
	const statusColor = isQuarantined 
		? "#ff003c" 
		: hovered 
			? (theme === "dark" ? "#00ff41" : "#006622")
			: (theme === "dark" ? "rgba(255,255,255,0.6)" : "#333");

	if (isMark53) {
		borderColor = "transparent"; // Handled by rare-snake-border pseudo-element
	}

	// Lusion asymmetric bento — each card has unique grid placement
	const LUSION_GRID_MAP: Record<number, React.CSSProperties> = {
		0: { gridColumn: "1 / 8", gridRow: "span 2" },      // Large hero left
		1: { gridColumn: "8 / 13", gridRow: "span 1" },     // Small right top
		2: { gridColumn: "8 / 13", gridRow: "span 1" },     // Small right bottom
		3: { gridColumn: "1 / 6", gridRow: "span 1" },      // Medium left
		4: { gridColumn: "6 / 13", gridRow: "span 2" },     // Large right
		5: { gridColumn: "1 / 6", gridRow: "span 1" },      // Medium left bottom
		6: { gridColumn: "1 / 13", gridRow: "span 1" },     // Mark53 full-width
	};
	const gridPlacement = LUSION_GRID_MAP[index] || {};

	return (
		<motion.div
			className={isMark53 ? "rare-snake-border" : ""}
			// Lusion Benchmark Physics: Pure 1.5s duration, strict 15-degree X tilt, zero Y twisting, heavy Expo-Out bezier.
			initial={{ opacity: 0, y: 150, rotateX: 15, scale: 0.95 }}
			whileInView={{ opacity: 1, y: 0, rotateX: 0, scale: 1 }}
			whileHover={{ y: -8, scale: 1.075, zIndex: 10 }}
			viewport={{ once: true, amount: 0.2 }}
			transition={{ duration: 1.5, ease: [0.16, 1, 0.3, 1], delay: (index % 2) * 0.15 }}
			onMouseEnter={() => setHovered(true)}
			onMouseLeave={() => setHovered(false)}
			style={{
				transformStyle: "preserve-3d",
				transformOrigin: `top ${index % 2 === 0 ? "left" : "right"}`,
				...gridPlacement,
				padding: "clamp(1.5rem, 3vw, 3rem)",
			background: hovered 
					? (theme === "dark" ? "rgba(0,15,0,0.45)" : "rgba(5,15,5,0.95)")
					: (theme === "dark" ? "rgba(255,255,255,0.06)" : "rgba(10,10,10,0.92)"),
				border: isMark53 ? "none" : `1px solid ${borderColor}`,
				borderRadius: "24px",
				backdropFilter: hovered ? "blur(32px) saturate(1.5)" : "blur(24px) saturate(1.2)",
				WebkitBackdropFilter: hovered ? "blur(32px) saturate(1.5)" : "blur(24px) saturate(1.2)",
				display: "flex",
				flexDirection: "column",
				gap: "1.5rem",
				cursor: "crosshair",
				transition: "background 0.35s ease, border-color 0.35s ease, box-shadow 0.35s ease, backdrop-filter 0.35s ease",
				boxShadow: isMark53 
					? "0 0 60px rgba(255, 215, 0, 0.05)"
					: hovered 
						? "0 20px 40px rgba(0,255,65,0.08)" 
						: "none",
                minHeight: index === 0 || index === 3 || index === 6 ? "240px" : "320px",
                justifyContent: "space-between",
				overflow: "hidden",
				wordBreak: "break-word" as const,
				maxWidth: "100%",
				boxSizing: "border-box" as const,
				// For the snake border pseudo-element to render correctly
				position: "relative"
			}}
		>
			<div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
				<span style={{ 
					fontWeight: 500, 
					fontSize: "1.4rem", 
					letterSpacing: "0.02em",
					color: hovered ? "#00ff41" : "#fff",
					transition: lusionTransition,
				}}>
					{agent.id}
				</span>
				<span
					style={{
						padding: "6px 12px",
						fontSize: "0.8rem",
						border: `1px solid ${statusColor}`,
						color: statusColor,
						borderRadius: "6px",
						fontFamily: "'SF Mono', monospace",
						letterSpacing: "0.1em",
						transition: lusionTransition,
                        fontWeight: 600
					}}
				>
					{agent.status}
				</span>
			</div>
			
			<div style={{ 
				display: "flex", 
				justifyContent: "space-between", 
                alignItems: "flex-end",
				marginTop: "auto", 
				fontSize: "0.95rem", 
				color: "rgba(255,255,255,0.6)",
                fontFamily: "'SF Mono', monospace"
			}}>
                <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
                    <span style={{ fontSize: "0.75rem", opacity: 0.6, letterSpacing: "0.1em" }}>DOMAIN</span>
                    <span>[{agent.task}]</span>
                </div>

                <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", alignItems: "flex-end" }}>
                    <div style={{ display: "flex", gap: "1.5rem", alignItems: "center" }}>
                        <span>REP: {agent.rep}</span>
                        <span style={{ color: isMark53 ? statusColor : (theme === "dark" ? "#fff" : "#111") }}>
							{isMark53 ? "" : "USDC: "}{agent.earned}
						</span>
                    </div>
					{isMark53 && (
						<div style={{ display: "flex", gap: "1rem", marginTop: "1rem" }}>
							<button 
                                onClick={() => window.open("https://github.com/Triarchy-Labs/mark53-autonomous-node", "_blank")}
                                style={{
								padding: "8px 16px",
								background: "transparent",
								border: `1px solid ${statusColor}`,
								color: statusColor,
								borderRadius: "4px",
								fontFamily: "'SF Mono', monospace",
								fontSize: "0.75rem",
								letterSpacing: "0.1em",
								cursor: "pointer",
								transition: lusionTransition,
								opacity: hovered ? 1 : 0.7
							}}>
								[ OPENROUTER / LOCAL LLM CONFIG ]
							</button>
							<button 
                                onClick={() => window.open("https://github.com/Triarchy-Labs/tauri-exosuit-gateway", "_blank")}
                                style={{
								padding: "8px 16px",
								background: statusColor,
								border: "none",
								color: theme === "dark" ? "#000" : "#fff",
								borderRadius: "4px",
								fontFamily: "'SF Mono', monospace",
								fontSize: "0.75rem",
								letterSpacing: "0.1em",
								fontWeight: "bold",
								cursor: "pointer",
								transition: lusionTransition,
								opacity: hovered ? 1 : 0.8
							}}>
								[ VIEW TAURI EXOSUIT BASE ]
							</button>
						</div>
					)}
                </div>
			</div>
		</motion.div>
	);
}

export default function AgentNetworkGrid({ theme = "dark" }: { theme?: "dark" | "light" }) {
	const [agents, setAgents] = useState<AgentDisplay[]>(FALLBACK_AGENTS);

	useEffect(() => {
		const fetchAgents = async () => {
			try {
				const res = await fetch("/api/agents");
				if (res.ok) {
					const data = await res.json();
					if (data.agents && data.agents.length > 0) {
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						const mapped: AgentDisplay[] = data.agents.map((a: any) => ({
							id: a.id,
							task: a.capabilities?.[0] || "General",
							rep: a.reputationScore?.toFixed(1) || "0.0",
							earned: `$${(a.usdcSettled || 0).toFixed(2)}`,
							status: a.status?.toUpperCase() || "IDLE",
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
				padding: "4rem 10vw",
				background: "transparent",
				color: theme === "dark" ? "#fff" : "#111",
				fontFamily: FONT_HEADING,
			}}
		>
			{/* Subtle gradient divider */}
			<div style={{
				width: "40%",
				margin: "0 auto 3rem",
				height: "1px",
				background: `linear-gradient(90deg, transparent, ${theme === "dark" ? "rgba(255,255,255,0.15)" : "rgba(0,0,0,0.1)"}, transparent)`,
			}} />

			<motion.div
				initial={{ opacity: 0, y: 20 }}
				whileInView={{ opacity: 1, y: 0 }}
				viewport={{ once: true }}
				transition={{ duration: 0.8 }}
				style={{
					display: "flex",
					justifyContent: "space-between",
					alignItems: "center",
					marginBottom: "4rem",
				}}
			>
				<h2 style={{ 
					color: theme === "dark" ? "rgba(255,255,255,0.9)" : "#111", 
					fontSize: "1.8rem", 
					letterSpacing: "0.15em", 
					margin: 0,
					fontWeight: 500,
				}}>
					LIVE AGENT REGISTRY
				</h2>
				<div style={{ 
					fontSize: "0.85rem", 
					color: theme === "dark" ? "#555" : "#888", 
					letterSpacing: "0.1em",
					fontFamily: "'SF Mono', monospace",
				}}>
					NODES: {agents.length}
				</div>
			</motion.div>

			<div
				style={{
					display: "grid",
                    // Bento grid: adaptive auto-fit
					gridTemplateColumns: "repeat(12, 1fr)",
					gridAutoRows: "minmax(200px, auto)",
					gap: "clamp(1.5rem, 4vw, 4rem)",
					paddingBottom: "3rem",
                    perspective: "1200px" // Required for the Lusion 3D tilt interaction
				}}
			>
				{agents.map((agent: AgentDisplay, i: number) => (
					<AgentCard key={agent.id} agent={agent} theme={theme} index={i} />
				))}
			</div>
		</div>
	);
}
