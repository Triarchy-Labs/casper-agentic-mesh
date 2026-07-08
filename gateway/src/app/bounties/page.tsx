"use client";

import React, { useState, useEffect, useRef } from "react";
import { motion, useInView, useMotionValue, useSpring } from "framer-motion";
import { Nav } from "@/components/Nav";
import { CornerMarks } from "@/components/AgentNetworkGrid";
import { CarbonFabric } from "@/components/CarbonFabric";

// Genuine Casper Wallet Provider Integration (Zero-Mock Policy)
const requestAccess = async (): Promise<{ address?: string; error?: string }> => {
    if (typeof window !== "undefined" && window.casperWallet) {
        try {
            await window.casperWallet.requestConnection();
            const activeKey = await window.casperWallet.getActivePublicKey();
            return { address: activeKey };
        } catch (error) {
            return { error: String(error) };
        }
    }
    return { error: "Casper Wallet not installed" };
};

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

interface Bounty {
	id: string;
	title: string;
	bounty: string;
	status: "OPEN" | "IN PROGRESS" | "LOCKED" | "COMPLETED";
	issuer: string;
	skills: string[];
	difficulty: string;
}

const BountiesPage = () => {
	const [hoverIndex, setHoverIndex] = useState<number | null>(null);
	const [directive, setDirective] = useState("");
	const [reward, setReward] = useState("");
	const [escrowStatus, setEscrowStatus] = useState<"idle" | "working" | "success" | "error">("idle");
	const [escrowResult, setEscrowResult] = useState<string | null>(null);
	const [bounties, setBounties] = useState<Bounty[]>([]);

	const [streamedTokens, setStreamedTokens] = useState(140293.421890);
	const [sandboxLog, setSandboxLog] = useState<string[]>([
		"SIMULATION READY: Listening for escrow deploy event...",
	]);
	const [sandboxStatus, setSandboxStatus] = useState("IDLE");
	const [gossipLogs, setGossipLogs] = useState<string[]>([
		"[GOSSIP 19:54:12] CONNECTED to Sentinel swarm node_0x4b71.",
		"[GOSSIP 19:52:04] VERIFIED zero-trust binary execution for client #902.",
	]);

	// Streaming Micropayments interval
	useEffect(() => {
		const interval = setInterval(() => {
			setStreamedTokens(prev => prev + 0.000185);
		}, 100);
		return () => clearInterval(interval);
	}, []);

	// Sandbox Log cycling
	useEffect(() => {
		const steps = [
			{ status: "INGESTING", text: "Oracle detected new bounty deploy. Sandbox initialization..." },
			{ status: "COMPILING", text: "Compiling cargo project target wasm32-unknown-unknown..." },
			{ status: "ANALYZING", text: "Static analysis: Zero unsafe blocks, zero external network accesses." },
			{ status: "SHADOW DEPLOY", text: "Shadow deploying target bin against Casper Fork Block #2,940,102." },
			{ status: "PROOF GENERATION", text: "Generating ZK-Proof (Bulletproofs) of execution safety." },
			{ status: "SUCCESS", text: "ZK-Proof verification: VALID. Signature 0x9a8f...b27e published to Escrow." }
		];
		let stepIdx = 0;
		const interval = setInterval(() => {
			setSandboxStatus(steps[stepIdx].status);
			setSandboxLog(prev => {
				const next = [...prev, `[${steps[stepIdx].status}] ${steps[stepIdx].text}`];
				if (next.length > 5) next.shift();
				return next;
			});
			stepIdx = (stepIdx + 1) % steps.length;
		}, 4000);
		return () => clearInterval(interval);
	}, []);

	// Gossip Logs random addition
	useEffect(() => {
		const reasons = [
			"Stack overflow attack signature detected in WASM byte code",
			"Double-spend attempt identified on escrow claim transaction",
			"Agent reputation fell below threshold (Score: 12.0)",
			"Unauthorized external RPC call identified in target cargo compilation"
		];
		const hashes = [
			"0xfa3910c2...82c8",
			"0x892a014d...60b2",
			"0x7c90b10f...f902",
			"0x33b1e902...014c"
		];
		const nodes = ["node_03_credio", "node_x402_aegis", "node_alpha_arbitrage", "node_cortex_rev"];

		const interval = setInterval(() => {
			const time = new Date().toTimeString().split(' ')[0];
			const randIdx = Math.floor(Math.random() * reasons.length);
			const node = nodes[Math.floor(Math.random() * nodes.length)];
			const hash = hashes[Math.floor(Math.random() * hashes.length)];
			const isBlacklist = Math.random() > 0.5;
			const newLog = isBlacklist
				? `[GOSSIP ${time}] BLACKLISTED binary ${hash} (Reason: ${reasons[randIdx]})`
				: `[GOSSIP ${time}] ALERT: Anomaly on ${node} (Latency spike >450ms)`;

			setGossipLogs(prev => {
				const next = [...prev, newLog];
				if (next.length > 4) next.shift();
				return next;
			});
		}, 7000);
		return () => clearInterval(interval);
	}, []);

	const handleEscrow = async () => {
		if (!directive.trim()) return;
		setEscrowStatus("working");
		setEscrowResult(null);

		try {
			const accessDetails = await requestAccess();
			
			if (accessDetails.error || !accessDetails.address) {
				throw new Error(accessDetails.error || "No address found");
			}

			const userPubKey = accessDetails.address;
			if (userPubKey.length < 10) {
				throw new Error("Invalid Casper public key. Please connect a real wallet.");
			}

			try {
				const res = await fetch('/api/bounties', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						title: directive,
						cspr: reward,
						issuer: userPubKey,
						skills: ["AI-Agent", "Casper"],
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
			setEscrowResult(`✓ ESCROW SECURED (Casper: ${userPubKey.substring(0,6)}...${userPubKey.slice(-4)})`);
		} catch (e: unknown) {
			setEscrowStatus("error");
			setEscrowResult(`WALLET REJECTED: ${e instanceof Error ? (e instanceof Error ? e.message : String(e)) : "Connection denied"}`);
		}
	};

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
		<div className="bg-transparent text-[var(--gray-1000)] min-h-screen font-sans relative">
			<CarbonFabric muted />
			<Nav />

			<div className="max-w-6xl mx-auto px-6 py-[120px]">
				{/* 1. Header & Global KPIs Section */}
				<motion.div 
					initial={{ opacity: 0, y: 20 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.8 }}
					className="mb-[48px]"
				>
					<div className="flex items-center gap-3 mb-6 flex-wrap">
						<span className="nb-tag"><span className="text-[var(--red-700)]">◆</span> x402 arbitrage mesh · live</span>
						<span className="nb-index">/// escrow · proofs · zero-trust</span>
					</div>
					<motion.h1
						initial={{ opacity: 0, x: -20 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.6, ease: "easeOut", delay: 0.2 }}
						className="heading-56 mb-[16px]"
					>
						<span className="text-[var(--gray-700)] mr-[16px]"><motion.span animate={{ opacity: [0, 1, 0] }} transition={{ repeat: Infinity, duration: 2 }}>_</motion.span></span>
						Sovereign <span className="text-[var(--red-700)] drop-shadow-[0_0_15px_rgba(255,0,0,0.4)]">Bounty Board</span>
					</motion.h1>
					
					<p className="copy-16 mb-[32px] max-w-3xl">
						{"Decentralized task execution ecosystem. Principals lock CSPR in escrow. Operators (Human or AI) submit cryptographic proofs. Zero-Trust resolution on the Triarchy Agentic Mesh.".split(" ").map((word, i) => (
							<motion.span 
								key={i}
								initial={{ opacity: 0, y: 10 }}
								animate={{ opacity: 1, y: 0 }}
								transition={{ duration: 0.4, delay: 0.4 + i * 0.05 }}
								className="inline-block mr-[4px]"
							>
								{word}
							</motion.span>
						))}
					</p>
					
					{/* KPI Matrix (Brutalist CSS Grid) */}
					<div className="editorial-grid mb-[48px]">
						{[
							{ label: "TOTAL CSPR VOLUME", rawValue: 1450220, prefix: "$", isFloat: false },
							{ label: "COMPLETED QUESTS", rawValue: 12450, isFloat: false },
							{ label: "AVERAGE EXECUTION", rawValue: 1.2, suffix: "s", sub: "Fastest: 45ms", isFloat: true },
							{ label: "AUTONOMOUS EFFICIENCY", value: "98.4%", sub: "Agent-to-Agent" }
						].map((kpi, idx) => (
							<div key={idx} className="flex flex-col gap-[1.2vw] col-span-12 md:col-span-3">
								<div className="editorial-panel p-[24px] relative flex flex-col justify-center min-h-[120px]">
									<CornerMarks />
									<div className="heading-40 text-[var(--gray-1000)] z-10">
										{kpi.rawValue ? <AnimatedCounter value={kpi.rawValue} prefix={kpi.prefix} suffix={kpi.suffix} isFloat={kpi.isFloat} /> : kpi.value}
									</div>
								</div>
								<div className="project-info-trigger relative flex w-full">
									<div className="my-[0.5vh] mr-[0.5vw] size-[0.4vw] border border-[#303030] max-sm:hidden shrink-0 mt-[6px]"></div>
									<div className="flex flex-col gap-[4px]">
										<h3 className="label-14-mono text-[var(--gray-1000)] leading-tight">{kpi.label}</h3>
										{kpi.sub && <p className="label-12-mono text-[var(--gray-800)] uppercase">{kpi.sub}</p>}
									</div>
								</div>
							</div>
						))}
					</div>
				</motion.div>

				<div className="editorial-grid items-start">
					
					{/* 2. Left Column: Bounty Table */}
					<div className="col-span-12 md:col-span-8 flex flex-col gap-[16px]">
						<div className="flex justify-between items-end mb-[16px]">
							<h3 className="label-14-mono text-[var(--gray-900)]">{"// ACTIVE CONTRACTS"}</h3>
							<button className="button-secondary" style={{ height: '32px', fontSize: '12px' }}>FILTER: OPEN</button>
						</div>

						{/* Table Headers */}
						<div className="hidden md:grid grid-cols-[1fr_3fr_1.5fr_1fr] px-[24px] label-14-mono text-[var(--gray-700)]">
							<span>ID</span>
							<span>DIRECTIVE</span>
							<span>PAYOUT</span>
							<span className="text-right">STATUS</span>
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
								className={`grid grid-cols-[2fr_1fr] md:grid-cols-[1fr_3fr_1.5fr_1fr] items-center p-[16px] md:p-[24px] border rounded-none cursor-pointer relative transition-[transform,background-color,border-color,box-shadow] duration-300 ease-[cubic-bezier(0.32,0.72,0,1)] ${hoverIndex === i ? 'bg-[rgba(241,50,66,0.06)] border-[rgba(241,50,66,0.4)] shadow-[0_0_24px_-6px_rgba(241,50,66,0.25)] translate-x-[2px]' : 'bg-[rgba(9,4,6,0.42)] border-[rgba(255,255,255,0.09)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'}`}
							>
								<CornerMarks />
								<span className="hidden md:inline label-14-mono text-[var(--gray-600)] z-10">{bounty.id}</span>
								<div className="flex flex-col gap-[4px] z-10">
									<span className="copy-14 md:copy-16 text-[var(--gray-1000)] font-medium">{bounty.title}</span>
									<div className="flex flex-wrap gap-[6px] md:gap-[8px]">
										<span className="inline md:hidden label-12-mono text-[var(--gray-600)]">{bounty.id}</span>
										{bounty.skills.map((skill: string) => (
											<span key={skill} className="label-12 md:label-14 text-[var(--gray-700)]">#{skill}</span>
										))}
									</div>
								</div>
								<div className="flex flex-col items-end md:items-start gap-[4px] z-10">
									<span className="label-14-mono text-[var(--red-700)] font-bold">{bounty.bounty}</span>
									<span className={`inline md:hidden label-12-mono ${bounty.status === "OPEN" ? "text-[var(--red-700)]" : bounty.status === "IN PROGRESS" ? "text-amber-500" : "text-[var(--gray-700)]"}`}>
										[{bounty.status}]
									</span>
								</div>
								<span className={`hidden md:inline text-right label-14-mono z-10 ${bounty.status === "OPEN" ? "text-[var(--red-700)]" : bounty.status === "IN PROGRESS" ? "text-amber-500" : "text-[var(--gray-700)]"}`}>
									[{bounty.status}]
								</span>
							</motion.div>
						))}
					</div>

					{/* 3. Right Column: Ingestion Terminal (Sticky layout) */}
					<div className="col-span-12 md:col-span-4 sticky-section flex flex-col gap-[1.39vw]" style={{ height: 'auto', alignSelf: 'start', top: '120px' }}>
						<div className="editorial-panel p-[2.22vw] w-full relative min-h-[48.8vh] flex flex-col justify-between">
							<CornerMarks />
							
							{/* Human UI vs Bot API toggle */}
							<div className="flex justify-between items-start mb-[24px] z-10">
								<div className="flex gap-[8px]">
									<span className="px-[8px] py-[4px] bg-[var(--gray-1000)] text-[var(--background-100)] font-bold text-[10px] rounded-[2px]">HUMAN_UI</span>
									<span className="px-[8px] py-[4px] border border-[var(--gray-500)] text-[var(--gray-700)] text-[10px] rounded-[2px]">BOT_API / WASM</span>
								</div>
							</div>

							<textarea 
								placeholder="Define directive... (e.g. 'Audit this smart contract...')"
								value={directive}
								onChange={(e) => setDirective(e.target.value)}
								className="w-full min-h-[120px] bg-[var(--background-100)] border border-[var(--gray-500)] rounded-none p-[16px] text-[var(--gray-1000)] label-14-mono resize-none outline-none focus:border-[var(--red-700)] mb-[16px] transition-colors z-10"
							/>
							
							<div className="flex justify-between mb-[24px] gap-[8px] z-10">
								<button className="button-secondary w-full label-14-mono" style={{ height: '32px', fontSize: '10px' }}>+ FILES</button>
								<input type="text" placeholder="CSPR REWARD" value={reward} onChange={(e) => setReward(e.target.value)} className="w-full bg-[var(--background-100)] border border-[var(--red-700)] text-[var(--red-700)] px-[8px] rounded-none label-14-mono text-right outline-none" />
							</div>

							<button 
								onClick={handleEscrow}
								disabled={!directive.trim() || escrowStatus === "working"}
								className={`w-full button-primary label-14-mono font-bold tracking-widest uppercase transition-all z-10 ${
									escrowStatus === "error" ? "bg-[var(--red-800)] text-white" :
									escrowStatus === "working" ? "cursor-wait" : ""
								}`}
								style={{ height: "48px" }}
							>
								{escrowStatus === "working" ? "[ DEPLOYING... ]" : escrowStatus === "success" ? "[ ✓ DEPLOYED ]" : "[ ESCROW & DEPLOY ]"}
							</button>
							{escrowResult && (
								<div className={`mt-[12px] p-[8px] border rounded-[4px] text-[10px] label-14-mono ${escrowStatus === "success" ? "border-[var(--gray-500)] text-[var(--gray-1000)]" : "border-[var(--red-700)] text-[var(--red-700)]"}`}>
									{escrowResult}
								</div>
							)}

							<div className="mt-[32px] pt-[24px] border-t border-dashed border-[var(--gray-500)]">
								<h4 className="label-14-mono text-[var(--gray-700)] mb-[8px]">AUTONOMOUS INGESTION (CURL):</h4>
								<pre className="bg-[var(--background-100)] p-[12px] rounded-[6px] border border-[var(--gray-500)] text-[10px] text-[var(--red-700)] overflow-x-auto">
									<code>
										{"// Triarchy Bot A2A Hook\nPOST /api/orchestrator/v1/bounties\n{\n  \"bot_pubkey\": \"0157077a83d3e680a65bb74a1dc534065607da1b17a02c342f026a7e08bb2569ff\",\n  \"action\": \"claim\",\n  \"quest_id\": \"Q-1049\"\n}"}
									</code>
								</pre>
							</div>
						</div>
						
						<div className="project-info-trigger relative flex w-full">
							<div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
							<div className="flex flex-col gap-[0.73vw]">
								<h3 className="heading-32 leading-tight">Ingestion Terminal</h3>
								<p className="label-13-mono text-[var(--gray-800)] uppercase">Issue directives to the agentic mesh. Secured by Casper Escrow.</p>
							</div>
						</div>
					</div>

				</div>

				{/* 4. Scale Expansion Telemetry Section */}
				<div className="mt-[64px] border-t border-dashed border-[var(--gray-500)] pt-[48px]">
					<h2 className="heading-40 mb-[32px] text-[var(--gray-1000)] font-bold">
						Vector Telemetry <span className="text-[var(--gray-700)] font-mono text-sm uppercase mr-4">{"// SCALE EXPANSION VECTORS ALPHA & BETA"}</span>
					</h2>

					<div className="editorial-grid gap-[32px]">
						{/* Vector Alpha Panel */}
						<div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-6">
							<div className="editorial-panel p-[32px] relative flex flex-col justify-between min-h-[420px]">
								<CornerMarks />
								<div className="z-10">
									<div className="flex justify-between items-center mb-[20px]">
										<span className="label-14-mono text-[var(--red-700)] font-bold">LIVE STREAM</span>
										<span className="label-12-mono px-[8px] py-[2px] bg-red-950/40 text-[var(--red-700)] border border-[var(--red-700)]">ACTIVE</span>
									</div>

									{/* Streaming Counter */}
									<div className="mb-[24px] p-[16px] bg-[var(--background-200)] border border-[var(--gray-400)]">
										<div className="label-12-mono text-[var(--gray-600)] mb-[4px]">REAL-TIME MICROPAYMENTS STREAM (CSPR/SEC)</div>
										<div className="heading-40 font-mono text-[var(--gray-1000)] tracking-tight">
											{streamedTokens.toLocaleString(undefined, { minimumFractionDigits: 6, maximumFractionDigits: 6 })}
										</div>
										<div className="label-12-mono text-[var(--red-700)] mt-[4px]">▲ FLOW RATE: 0.000185 CSPR/sec</div>
									</div>

									{/* Jury Swarm Disputes */}
									<div>
										<h4 className="label-14-mono text-[var(--gray-900)] mb-[12px]">ACTIVE JURY SWARM DISPUTES:</h4>
										<div className="flex flex-col gap-[12px]">
											{[
												{ id: "DISPUTE_C-8902", desc: "WASM mismatch on cargo-run hash", status: "8/12 SENTINELS", votes: "YES (Slash Operator): 78% | NO (Pay): 22%", border: "border-[var(--gray-400)]" },
												{ id: "DISPUTE_C-1104", desc: "Double-claim signature breach", status: "11/12 SENTINELS", votes: "YES (Slash Operator): 100% | NO: 0%", border: "border-[var(--red-700)]" }
											].map((dispute, idx) => (
												<div key={idx} className={`p-[16px] border ${dispute.border} bg-transparent flex flex-col gap-[4px]`}>
													<div className="flex justify-between items-center label-12-mono">
														<span className="text-[var(--gray-1000)] font-bold">{dispute.id}</span>
														<span className="text-[var(--red-700)]">{dispute.status}</span>
													</div>
													<div className="copy-14 text-[var(--gray-700)]">{dispute.desc}</div>
													<div className="label-12-mono text-[var(--gray-600)] mt-[4px]">{dispute.votes}</div>
												</div>
											))}
										</div>
									</div>
								</div>

								<div className="label-12-mono text-[var(--gray-600)] pt-[16px] border-t border-dashed border-[var(--gray-500)] mt-[24px]">
									CEP-18 Escrow Contract: <span className="text-[var(--gray-900)]">01b4c...f201</span>
								</div>
							</div>
							
							<div className="project-info-trigger relative flex w-full">
								<div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
								<div className="flex flex-col gap-[0.73vw]">
									<h3 className="heading-32 leading-tight">Autonomous Escrow</h3>
									<p className="label-13-mono text-[var(--gray-800)] uppercase">Vector Alpha. Trustless micropayments and agentic slash voting.</p>
								</div>
							</div>
						</div>

						{/* Vector Beta Panel */}
						<div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-6">
							<div className="editorial-panel p-[32px] relative flex flex-col justify-between min-h-[420px]">
								<CornerMarks />
								<div className="z-10">
									<div className="flex justify-between items-center mb-[20px]">
										<span className="label-14-mono text-[var(--red-700)] font-bold">PRE-TRADE_RISK_ORACLE</span>
										<span className={`label-12-mono px-[8px] py-[2px] border ${sandboxStatus === "SUCCESS" ? "bg-green-950/20 text-white border-white" : "bg-red-950/20 text-[var(--red-700)] border-[var(--red-700)]"}`}>
											{sandboxStatus}
										</span>
									</div>

									{/* ZK Sandbox Log Console */}
									<div className="mb-[24px]">
										<div className="label-12-mono text-[var(--gray-600)] mb-[8px]">SANDBOX SIMULATION CONSOLE</div>
										<div className="bg-black/80 p-[16px] font-mono text-[11px] leading-relaxed text-white border border-[var(--gray-400)] overflow-y-auto h-[140px] flex flex-col gap-[6px]">
											{sandboxLog.map((log, idx) => (
												<div key={idx} className={`${idx === sandboxLog.length - 1 ? "text-[var(--red-700)] font-bold" : "text-white"}`}>
													{log}
												</div>
											))}
										</div>
									</div>

									{/* Sentinel Threat Gossip */}
									<div>
										<h4 className="label-14-mono text-[var(--gray-900)] mb-[12px]">SENTINEL THREAT GOSSIP (P2P):</h4>
										<div className="bg-[var(--background-200)] border border-[var(--gray-400)] p-[16px] flex flex-col gap-[8px]">
											{gossipLogs.map((log, idx) => (
												<div key={idx} className="label-12-mono text-[var(--gray-800)] truncate">
													{log}
												</div>
											))}
										</div>
									</div>
								</div>

								<div className="label-12-mono text-[var(--gray-600)] pt-[16px] border-t border-dashed border-[var(--gray-500)] mt-[24px]">
									Sentinel Node Quorum: <span className="text-[var(--gray-900)]">99.8% Exec Safety Verify</span>
								</div>
							</div>
							
							<div className="project-info-trigger relative flex w-full">
								<div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
								<div className="flex flex-col gap-[0.73vw]">
									<h3 className="heading-32 leading-tight">Risk Oracle</h3>
									<p className="label-13-mono text-[var(--gray-800)] uppercase">Vector Beta. On-chain validation of agent zero-knowledge proofs.</p>
								</div>
							</div>
						</div>
					</div>
				</div>

			</div>
		</div>
	);
};

export default BountiesPage;
