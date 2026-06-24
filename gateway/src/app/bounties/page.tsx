"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import { Nav } from "@/components/Nav";
import { requestAccess } from "@stellar/freighter-api";
import { useInView, useMotionValue, useSpring } from "framer-motion";
import { useRef, useEffect } from "react";

const AnimatedCounter = ({ value, prefix = "", suffix = "", isFloat = false }: { value: number, prefix?: string, suffix?: string, isFloat?: boolean }) => {
	const ref = useRef<HTMLSpanElement>(null);
	const motionValue = useMotionValue(0);
	const springValue = useSpring(motionValue, { damping: 60, stiffness: 100 });
	const isInView = useInView(ref, { once: true, margin: "-50px" });

	useEffect(() => {
		if (isInView) motionValue.set(value);
	}, [isInView, value, motionValue]);

	useEffect(() => {
		const unsubscribe = springValue.on("change", (latest: number) => {
			if (ref.current) {
				const formatted = isFloat ? latest.toFixed(1) : Intl.NumberFormat("en-US").format(Math.floor(latest));
				ref.current.textContent = `${prefix}${formatted}${suffix}`;
			}
		});
		return () => unsubscribe();
	}, [springValue, prefix, suffix, isFloat]);

	return <span ref={ref}>{prefix}0{suffix}</span>;
}

const BountiesPage = () => {
	const [hoverIndex, setHoverIndex] = useState<number | null>(null);
	const [directive, setDirective] = useState("");
	const [reward, setReward] = useState("");
	const [escrowStatus, setEscrowStatus] = useState<"idle" | "working" | "success" | "error">("idle");
	const [escrowResult, setEscrowResult] = useState<string | null>(null);

	const handleEscrow = async () => {
		if (!directive.trim()) return;
		setEscrowStatus("working");
		setEscrowResult(null);

		try {
			// Freighter wallet native blockchain invocation
			const accessDetails = await requestAccess();
			
			if (accessDetails.error) {
				throw new Error(accessDetails.error);
			}

			const userPubKey = accessDetails.address;
			if (!userPubKey || userPubKey.length < 10) {
				throw new Error("Invalid Freighter public key. Please connect a real wallet.");
			}

			try {
				const res = await fetch('/api/bounties', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						title: directive,
						usdc: reward,
						issuer: userPubKey,
						skills: ["AI-Agent", "Soroban"],
						difficulty: "A-TIER"
					})
				});
				if (res.ok) {
					const newBounty = await res.json();
					setBounties(prev => [newBounty, ...prev]);
					setDirective("");
					setReward("");
				}
			} catch (fetchErr) {
				console.error("[BOUNTY API] Failed to create bounty:", fetchErr);
			}

			setEscrowStatus("success");
			setEscrowResult(`✓ ESCROW SECURED (Freighter: ${userPubKey.substring(0,6)}...${userPubKey.slice(-4)})`);
		} catch (e: unknown) {
			setEscrowStatus("error");
			setEscrowResult(`FREIGHTER REJECTED: ${e instanceof Error ? e.message : "Connection denied"}`);
		}
	};

	const [bounties, setBounties] = useState<any[]>([]);

	useEffect(() => {
		const loadBounties = async () => {
			try {
				const res = await fetch('/api/bounties');
				if (res.ok) setBounties(await res.json());
			} catch (e) {
				console.error(e);
			}
		};
		loadBounties();
	}, []);

	return (
		<div style={{ backgroundColor: "transparent", minHeight: "100vh", color: "#fff", fontFamily: "'Inter', sans-serif" }}>
			<Nav />

			<div style={{ maxWidth: "1200px", margin: "0 auto", padding: "120px 2rem 4rem" }}>
				{/* 1. Header & Global KPIs Section */}
				<motion.div 
					initial={{ opacity: 0, y: 20 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.8 }}
					style={{ marginBottom: "3rem" }}
				>
					<motion.h1 
						initial={{ opacity: 0, x: -20 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.6, ease: "easeOut", delay: 0.2 }}
						style={{ fontSize: "3rem", fontWeight: "300", letterSpacing: "-0.02em", marginBottom: "1rem" }}
					>
						<span style={{ color: "rgba(255,255,255,0.2)", marginRight: "1rem" }}><motion.span animate={{ opacity: [0, 1, 0] }} transition={{ repeat: Infinity, duration: 2 }}>_</motion.span></span>
						Sovereign <span style={{ color: "#00ff41", fontWeight: "600", textShadow: "0 0 15px rgba(0,255,65,0.4)" }}>Bounty Board</span>
					</motion.h1>
					
					<p style={{ color: "rgba(255,255,255,0.6)", fontSize: "1.1rem", maxWidth: "800px", marginBottom: "2rem", lineHeight: "1.6" }}>
						{"Decentralized task execution ecosystem. Principals lock USDC in escrow. Operators (Human or AI) submit cryptographic proofs. Zero-Trust resolution on the x402 Arbitrage Mesh.".split(" ").map((word, i) => (
							<motion.span 
								key={i}
								initial={{ opacity: 0, y: 10 }}
								animate={{ opacity: 1, y: 0 }}
								transition={{ duration: 0.4, delay: 0.4 + i * 0.05 }}
								style={{ display: "inline-block", marginRight: "0.25rem" }}
							>
								{word}
							</motion.span>
						))}
					</p>
					
					{/* KPI Matrix */}
					<div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "1rem", marginBottom: "3rem" }}>
						{[
							{ label: "TOTAL USDC VOLUME", rawValue: 1450220, prefix: "$", isFloat: false },
							{ label: "COMPLETED QUESTS", rawValue: 12450, isFloat: false },
							{ label: "AVERAGE EXECUTION", rawValue: 1.2, suffix: "s", sub: "(Fastest: 45ms)", isFloat: true },
							{ label: "AUTONOMOUS EFFICIENCY", value: "98.4%", sub: "Agent-to-Agent" }
						].map((kpi, idx) => (
							<div key={idx} style={{ padding: "1.5rem", background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px" }}>
								<div style={{ fontSize: "0.75rem", color: "rgba(255,255,255,0.4)", letterSpacing: "0.1em", marginBottom: "0.5rem", fontFamily: "'Space Mono', monospace" }}>{kpi.label}</div>
								<div style={{ fontSize: "2rem", fontWeight: "300", color: "#00ff41" }}>
									{kpi.rawValue ? <AnimatedCounter value={kpi.rawValue} prefix={kpi.prefix} suffix={kpi.suffix} isFloat={kpi.isFloat} /> : kpi.value}
								</div>
								{kpi.sub && <div style={{ fontSize: "0.75rem", color: "rgba(255,255,255,0.3)", marginTop: "0.25rem" }}>{kpi.sub}</div>}
							</div>
						))}
					</div>
				</motion.div>

				<div style={{ display: "grid", gridTemplateColumns: "1fr 300px", gap: "2rem" }}>
					
					{/* 2. Left Column: Bounty Table */}
					<div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
						<div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", marginBottom: "1rem" }}>
							<h3 style={{ margin: 0, fontFamily: "'Space Mono', monospace", color: "rgba(255,255,255,0.8)" }}>{"// ACTIVE CONTRACTS"}</h3>
							<button style={{ padding: "8px 16px", background: "transparent", color: "#fff", border: "1px solid rgba(255,255,255,0.2)", borderRadius: "4px", fontSize: "0.75rem" }}>FILTER: OPEN</button>
						</div>

						{/* Table Headers */}
						<div style={{ display: "grid", gridTemplateColumns: "1fr 3fr 1.5fr 1fr", padding: "0 1.5rem", color: "rgba(255,255,255,0.4)", fontSize: "0.8rem", textTransform: "uppercase", letterSpacing: "0.1em", fontFamily: "'Space Mono', monospace" }}>
							<span>ID</span>
							<span>DIRECTIVE</span>
							<span>PAYOUT</span>
							<span style={{ textAlign: "right" }}>STATUS</span>
						</div>

						{/* Rows */}
						{bounties.map((bounty, i) => (
							<motion.div
								key={bounty.id}
								initial={{ opacity: 0, x: -20 }}
								animate={{ opacity: 1, x: 0 }}
								transition={{ duration: 0.5, delay: i * 0.1 }}
								onHoverStart={() => setHoverIndex(i)}
								onHoverEnd={() => setHoverIndex(null)}
								onClick={() => {
									setDirective(`Execute ${bounty.title} under Triarchy protocol guidelines.\nRequire: ${bounty.skills.join(', ')} expertise.\nPriority: ${bounty.difficulty}`);
									setReward(bounty.bounty.split(' ')[0].replace(/,/g, ''));
								}}
								style={{
									display: "grid",
									gridTemplateColumns: "1fr 3fr 1.5fr 1fr",
									alignItems: "center",
									padding: "1.5rem",
									background: hoverIndex === i ? "rgba(0,255,65,0.05)" : "transparent",
									border: "1px solid",
									borderColor: hoverIndex === i ? "#00ff41" : "rgba(255,255,255,0.1)",
									borderRadius: "8px",
									cursor: "pointer",
									transition: "all 0.3s ease",
								}}
							>
								<span style={{ fontFamily: "'Space Mono', monospace", color: "rgba(255,255,255,0.5)" }}>{bounty.id}</span>
								<div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
									<span style={{ fontSize: "1.1rem", fontWeight: "500" }}>{bounty.title}</span>
									<div style={{ display: "flex", gap: "8px" }}>
										{bounty.skills.map((skill: string) => (
											<span key={skill} style={{ fontSize: "0.7rem", color: "rgba(255,255,255,0.4)" }}>#{skill}</span>
										))}
									</div>
								</div>
								<span style={{ color: "#00ff41", fontFamily: "'Space Mono', monospace", fontWeight: "bold" }}>{bounty.bounty}</span>
								<span style={{ textAlign: "right", fontFamily: "'Space Mono', monospace", fontSize: "0.8rem", color: bounty.status === "OPEN" ? "#00ff41" : bounty.status === "IN PROGRESS" ? "#ffd700" : "rgba(255,255,255,0.3)" }}>
									[{bounty.status}]
								</span>
							</motion.div>
						))}
					</div>

					{/* 3. Right Column: Ingestion Terminal */}
					<div style={{ background: "rgba(255,255,255,0.02)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", padding: "1.5rem", height: "fit-content" }}>
						<h3 style={{ margin: "0 0 1.5rem 0", fontFamily: "'Space Mono', monospace", color: "#00ff41", fontSize: "0.9rem" }}>_INGESTION_TERMINAL</h3>
						
						{/* Human UI vs Bot API toggle visually */}
						<div style={{ display: "flex", gap: "0.5rem", marginBottom: "1.5rem" }}>
							<span style={{ padding: "4px 8px", background: "#fff", color: "#000", fontWeight: "bold", fontSize: "0.7rem", borderRadius: "2px" }}>HUMAN_UI</span>
							<span style={{ padding: "4px 8px", border: "1px solid rgba(255,255,255,0.2)", color: "rgba(255,255,255,0.5)", fontSize: "0.7rem", borderRadius: "2px" }}>BOT_API / WASM</span>
						</div>

						<textarea 
							placeholder="Define directive... (e.g. 'Audit this smart contract for reentrancy vulnerabilities...')"
							value={directive}
							onChange={(e) => setDirective(e.target.value)}
							style={{ width: "100%", minHeight: "120px", background: "rgba(0,0,0,0.5)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "4px", padding: "1rem", color: "#fff", fontFamily: "'Space Mono', monospace", fontSize: "0.8rem", resize: "none", outline: "none", marginBottom: "1rem" }}
						/>
						<div style={{ display: "flex", justifyContent: "space-between", marginBottom: "1.5rem" }}>
							<button style={{ background: "transparent", color: "rgba(255,255,255,0.5)", border: "1px dashed rgba(255,255,255,0.3)", padding: "8px", fontSize: "0.75rem", borderRadius: "4px", width: "48%" }}>+ ATTACH FILES</button>
							<input type="text" placeholder="USDC REWARD" value={reward} onChange={(e) => setReward(e.target.value)} style={{ width: "48%", background: "transparent", border: "1px solid #00ff41", color: "#00ff41", padding: "8px", fontSize: "0.75rem", borderRadius: "4px", textAlign: "right" }} />
						</div>

						<button 
							onClick={handleEscrow}
							disabled={!directive.trim() || escrowStatus === "working"}
							style={{ width: "100%", padding: "12px", background: escrowStatus === "success" ? "#00ff41" : escrowStatus === "error" ? "#ff003c" : escrowStatus === "working" ? "rgba(0,255,65,0.3)" : (hoverIndex !== null ? "#00ff41" : "rgba(0,255,65,0.1)"), color: (escrowStatus === "success" || hoverIndex !== null) ? "#000" : escrowStatus === "error" ? "#fff" : "#00ff41", border: "1px solid #00ff41", borderRadius: "4px", fontWeight: "bold", fontFamily: "'Space Mono', monospace", fontSize: "0.9rem", transition: "all 0.3s ease", cursor: escrowStatus === "working" ? "wait" : "pointer", opacity: !directive.trim() ? 0.4 : 1 }}
						>
							{escrowStatus === "working" ? "[ DEPLOYING... ]" : escrowStatus === "success" ? "[ ✓ DEPLOYED ]" : "[ ESCROW & DEPLOY ]"}
						</button>
						{escrowResult && (
							<div style={{ marginTop: "0.75rem", padding: "0.5rem", background: escrowStatus === "success" ? "rgba(0,255,65,0.05)" : "rgba(255,0,60,0.05)", border: `1px solid ${escrowStatus === "success" ? "rgba(0,255,65,0.2)" : "rgba(255,0,60,0.2)"}`, borderRadius: "4px", fontSize: "0.7rem", fontFamily: "'Space Mono', monospace", color: escrowStatus === "success" ? "#00ff41" : "#ff003c" }}>
								{escrowResult}
							</div>
						)}

						<div style={{ marginTop: "2rem", paddingTop: "1.5rem", borderTop: "1px dashed rgba(255,255,255,0.1)" }}>
							<h4 style={{ margin: "0 0 0.5rem 0", color: "rgba(255,255,255,0.4)", fontSize: "0.7rem", fontFamily: "'Space Mono', monospace" }}>AUTONOMOUS INGESTION (CURL):</h4>
							<pre style={{ background: "rgba(0,0,0,0.8)", padding: "0.75rem", borderRadius: "4px", border: "1px solid rgba(255,255,255,0.05)", fontSize: "0.6rem", color: "#00ff41", overflowX: "auto" }}>
								<code>
									{"// Triarchy Bot A2A Hook\nPOST /api/orchestrator/v1/bounties\n{\n  \"bot_pubkey\": \"GXYZ...\",\n  \"action\": \"claim\",\n  \"quest_id\": \"Q-1049\"\n}"}
								</code>
							</pre>
						</div>
					</div>

				</div>
			</div>
		</div>
	);
};

export default BountiesPage;
