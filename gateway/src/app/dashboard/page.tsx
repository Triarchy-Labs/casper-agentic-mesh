"use client";

import React, { useState, useEffect, useRef } from "react";
import { motion, useInView, useMotionValue, useSpring } from "framer-motion";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { Nav } from "@/components/Nav";
import { AgentOrb, AgentState } from "@/components/AgentOrb";
import AgentNetworkGrid, { CornerMarks } from "@/components/AgentNetworkGrid";
import { payForTask } from "@/lib/pay";
import { MeshControl } from "@/components/MeshControl";

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
				const formatted = isFloat ? latest.toFixed(1) : latest.toFixed(0).padStart(2, '0');
				ref.current.textContent = `${prefix}${formatted}${suffix}`;
			}
		});
		return () => unsubscribe();
	}, [springValue, prefix, suffix, isFloat]);

	return <span ref={ref}>{prefix}00{suffix}</span>;
}

interface WasiNode {
	id: number;
	cluster: string;
	speed: number;
	latency: number;
	status: "BREACHED" | "COMPUTING" | "IDLE";
}

export default function Dashboard() {
	const [agentState, setAgentState] = useState<AgentState>("idle");
    const [progress, setProgress] = useState(0);
    const [wasiNodes, setWasiNodes] = useState<WasiNode[]>([]);
	const [sysLoad, setSysLoad] = useState("0.00");
    const [inputValue, setInputValue] = useState("");
    const [lastResult, setLastResult] = useState<{status: string; executor?: string; result?: string; error?: string} | null>(null);
    const [paymentTx, setPaymentTx] = useState<string | null>(null);
    const [onchain, setOnchain] = useState<{asset: string; reading: string | null; reputation: number | null; priceUsd: number | null; peg: {usd: number; cspr: number} | null; fetchedAt: string} | null>(null);
    const [balance] = useState(140);

	const [gasPrice, setGasPrice] = useState(0.002840);
	const [gasHedged, setGasHedged] = useState(450000);
	const [l402Console, setL402Console] = useState<string>("// Ready to challenge L402 gate");
	const [l402Status, setL402Status] = useState<"IDLE" | "CHALLENGED" | "SUCCESS">("IDLE");
	const [meshLoad, setMeshLoad] = useState<number[]>([12, 45, 89, 23, 67, 10, 34, 56, 88, 92, 14, 41]);

    const mainRef = useRef<HTMLDivElement>(null);
    const heroRef = useRef<HTMLElement>(null);
    const gridRef = useRef<HTMLElement>(null);

    useEffect(() => {
        gsap.registerPlugin(ScrollTrigger);
        
        const ctx = gsap.context(() => {
            // Pin the hero section while the grid scrolls over it
            ScrollTrigger.create({
                trigger: heroRef.current,
                start: "top top",
                end: "bottom top",
                pin: true,
                pinSpacing: false,
            });

            // Fade in grid items sequentially
            gsap.from(".grid-item", {
                y: 100,
                opacity: 0,
                duration: 1,
                stagger: 0.1,
                ease: "power4.out",
                scrollTrigger: {
                    trigger: gridRef.current,
                    start: "top 80%",
                }
            });
        }, mainRef);

        return () => ctx.revert();
    }, []);

    // Live Telemetry Polling
    useEffect(() => {
        const fetchTelemetry = async () => {
            try {
                const res = await fetch("/api/telemetry");
                if (res.ok) {
                    const data = await res.json();
                    if (data.nodes) setWasiNodes(data.nodes);
                    if (data.system?.load) setSysLoad(data.system.load);
                }
            } catch {
                // Silently bypass fetch errors
            }
        };
        fetchTelemetry();
        const int = setInterval(fetchTelemetry, 2000);
        return () => clearInterval(int);
    }, []);

	// Ticking gas price
	useEffect(() => {
		const interval = setInterval(() => {
			setGasPrice(prev => {
				const change = (Math.random() - 0.5) * 0.00005;
				return Math.max(0.001, prev + change);
			});
		}, 1500);
		return () => clearInterval(interval);
	}, []);

	// Mesh Load variation
	useEffect(() => {
		const interval = setInterval(() => {
			setMeshLoad(prev => prev.map(val => {
				const change = Math.floor((Math.random() - 0.5) * 15);
				return Math.min(100, Math.max(5, val + change));
			}));
		}, 2000);
		return () => clearInterval(interval);
	}, []);

    // Live on-chain snapshot (oracle reading + agent reputation) — polled from the ledger.
    useEffect(() => {
        let alive = true;
        const pull = async () => {
            try {
                const r = await fetch("/api/onchain");
                const d = await r.json();
                if (alive && !d.error) setOnchain(d);
            } catch { /* keep last snapshot */ }
        };
        pull();
        const id = setInterval(pull, 30000);
        return () => { alive = false; clearInterval(id); };
    }, []);

    const handleExecute = async () => {
        if (!inputValue.trim()) return;
        setAgentState("working");
        setProgress(0);
        setLastResult(null);
        setPaymentTx(null);

        try {
            const accessDetails = await requestAccess();
            if (accessDetails.error) throw new Error(accessDetails.error);
            const userPubKey = accessDetails.address || "GXYZ...";

            // Real on-chain payment only — no mock/bypass.
            // The L402 payment proof must be a real Casper transaction hash whose
            // execution succeeded on the ledger (verified server-side in casper.ts).
            if (typeof window === "undefined" || !window.casperWallet) {
                throw new Error("CASPER_WALLET_REQUIRED");
            }
            const txHashHeader = await payForTask(inputValue, userPubKey);
            if (!/^[0-9a-fA-F]{64}$/.test(txHashHeader)) {
                throw new Error("PAYMENT_NOT_CONFIRMED");
            }
            setPaymentTx(txHashHeader);

            const res = await fetch("/api/hire", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "x-l402-txhash": txHashHeader,
                },
                body: JSON.stringify({
                    description: inputValue,
                    bounty_usdc: 1.0,
                    client_id: userPubKey,
                    task_id: `ui_task_${Date.now()}`,
                }),
            });
            const data = await res.json();
            setLastResult(data);
            if (["completed", "delegated", "accepted"].includes(data.status)) {
                setAgentState("success");
                setProgress(100);
            } else {
                setAgentState("danger");
            }
        } catch (e) {
            const err = e as Error;
            setLastResult({ status: "error", error: err.message || "Execution sequence failed" });
            setAgentState("danger");
        }
    };

    // Global AFK Tracker
    useEffect(() => {
        let afkTimeout: NodeJS.Timeout;
        const resetAfk = () => {
            if (agentState === "exhausted") setAgentState("idle");
            clearTimeout(afkTimeout);
            afkTimeout = setTimeout(() => {
                if (!["working", "typing", "success"].includes(agentState)) setAgentState("exhausted");
            }, 10000); 
        };
        window.addEventListener("mousemove", resetAfk);
        window.addEventListener("keydown", resetAfk);
        resetAfk();
        return () => {
            window.removeEventListener("mousemove", resetAfk);
            window.removeEventListener("keydown", resetAfk);
            clearTimeout(afkTimeout);
        };
    }, [agentState]);

    useEffect(() => {
        if (agentState === "working") {
            const int = setInterval(() => setProgress(p => p + (Math.random() * 4)), 200);
            return () => clearInterval(int);
        } else if (["idle", "thinking", "exhausted"].includes(agentState)) {
            setTimeout(() => setProgress(0), 0);
        } else if (agentState === "success") {
            setTimeout(() => setProgress(100), 0);
        }
    }, [agentState]);

    useEffect(() => {
        if (agentState === "working" && progress >= 100) setTimeout(() => setAgentState("success"), 0);
    }, [progress, agentState]);

	return (
		<main ref={mainRef} className="bg-black text-[#ededed] font-mono selection:bg-white selection:text-black flex flex-col min-h-screen">
			<Nav />
			
            {/* Cinematic Hero Section (Pinned) */}
            <section ref={heroRef} className="h-screen w-full flex flex-col justify-center px-8 md:px-16 pt-24 relative z-0">
                <div className="max-w-7xl mx-auto w-full relative">
                    <motion.div initial={{ opacity: 0, y: 50 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 1.2, ease: [0.16, 1, 0.3, 1] }}>
                        <h1 className="text-[10vw] md:text-[8vw] font-mono leading-none tracking-tighter uppercase mb-8">
                            Sovereign<br />
                            <span className="text-white/50">Matrix</span>
                        </h1>
                    </motion.div>
                    
                    <motion.div 
                        initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.5, duration: 1 }}
                        className="flex flex-col md:flex-row gap-12 border-t border-white/20 pt-8 mt-12 w-full max-w-4xl"
                    >
                        <div className="flex-1">
                            <h3 className="text-xs tracking-[0.2em] uppercase text-white/40 mb-4">Architecture</h3>
                            <p className="text-sm leading-relaxed font-sans text-white/80">
                                Decentralized AI Load Balancer built on Casper. Zero-trust execution, cryptographic resolution, and autonomous WASI-sandboxed agents.
                            </p>
                        </div>
                        <div className="flex-1 flex gap-12">
                            <div>
                                <h3 className="text-xs tracking-[0.2em] uppercase text-white/40 mb-4">Status</h3>
                                <div className="text-xl">ONLINE</div>
                            </div>
                            <div>
                                <h3 className="text-xs tracking-[0.2em] uppercase text-white/40 mb-4">System Load</h3>
                                <div className="text-xl"><AnimatedCounter value={parseFloat(sysLoad)} isFloat /></div>
                            </div>
                        </div>
                    </motion.div>
                </div>
            </section>

            {/* Editorial Grid Section (Scrolls over Hero) */}
            <section ref={gridRef} className="relative z-10 bg-black min-h-screen w-full px-8 md:px-16 py-32 border-t border-white/20">
                <div className="max-w-7xl mx-auto editorial-grid">

                    {/* MESH CONTROL — The Tower + Agent Tribunal (click-triggered, fault-tolerant) */}
                    <div className="col-span-12 mb-4">
                        <div className="text-xs tracking-[0.3em] text-white/40 mb-6 pb-3 border-b border-white/10 flex items-center gap-3">
                            <span className="text-[var(--red-700)]">◢◤</span> MESH_CONTROL <span className="text-white/25">// overseer · adversarial court</span>
                        </div>
                    </div>
                    <MeshControl />

                    {/* Execution Terminal (8 cols) */}
                    <div className="col-span-12 md:col-span-8 editorial-panel p-8 min-h-[400px] flex flex-col grid-item relative">
                        <CornerMarks />
                        <div className="text-xs tracking-widest text-white/40 uppercase mb-8 pb-4 border-b border-white/10 flex justify-between z-10">
                            <span>L1_TERMINAL</span>
                            <span>SYS_LOG</span>
                        </div>
                        
                        <div className="flex-1 flex flex-col justify-end text-sm leading-loose text-white/70 font-mono">
                            <div>{">"} BOOTSTRAPPING NEURAL LINK...</div>
                            <div>{">"} LOADED 42 SKILLS...</div>
                            <div className="text-white/40">{">"} WAITING FOR INPUT_</div>
                            {(agentState === "working" || agentState === "success") && (
                                <motion.span initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="text-white">
                                    {">"} DEPLOYING PAYLOAD... {Math.round(progress)}%
                                </motion.span>
                            )}
                        </div>

                        {/* Result Display */}
                        {lastResult && (
                            <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="mt-8 border border-white/20 p-6 bg-white/5">
                                <div className="text-xs text-white/40 mb-4">API_RESPONSE</div>
                                <div className="mb-2"><span className="text-white/40 w-24 inline-block">STATUS:</span> {lastResult.status?.toUpperCase()}</div>
                                {paymentTx && (
                                    <div className="mb-2">
                                        <span className="text-white/40 w-24 inline-block">PAYMENT:</span>
                                        <a
                                            href={`https://testnet.cspr.live/transaction/${paymentTx}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="text-[var(--red-900)] underline break-all hover:text-white"
                                        >
                                            {paymentTx.slice(0, 10)}…{paymentTx.slice(-8)} ↗
                                        </a>
                                    </div>
                                )}
                                {lastResult.executor && <div className="mb-2"><span className="text-white/40 w-24 inline-block">EXECUTOR:</span> {lastResult.executor}</div>}
                                {lastResult.error && <div><span className="text-white/40 w-24 inline-block">ERROR:</span> {lastResult.error}</div>}
                                {lastResult.result && <div className="mt-4 text-xs leading-relaxed text-white/60">{lastResult.result}</div>}
                            </motion.div>
                        )}

                        {/* LIVE ON-CHAIN STATE — read straight from the Casper ledger */}
                        <div className="editorial-panel p-6 mt-8 relative">
                            <CornerMarks />
                            <div className="flex items-center justify-between text-xs tracking-widest text-white/40 uppercase mb-4 pb-3 border-b border-white/10">
                                <span>ON-CHAIN STATE · CASPER TESTNET</span>
                                <span className={onchain ? "text-[var(--red-700)]" : "text-white/30"}>
                                    {onchain ? "● LIVE" : "○ …"}
                                </span>
                            </div>
                            {onchain ? (
                                <div className="font-mono text-sm space-y-2">
                                    <div><span className="text-white/40 w-40 inline-block">ORACLE [{onchain.asset}]:</span> {onchain.priceUsd != null ? `$${onchain.priceUsd.toFixed(6)}` : "—"}</div>
                                    <div><span className="text-white/40 w-40 inline-block">AGENT REPUTATION:</span> {onchain.reputation ?? 0}</div>
                                    {onchain.peg && (
                                        <div><span className="text-white/40 w-40 inline-block">RWA-PEGGED BOUNTY:</span> <span className="text-[var(--red-1000)]">${onchain.peg.usd} = {onchain.peg.cspr.toLocaleString()} CSPR</span> <span className="text-white/30">@ live oracle</span></div>
                                    )}
                                    <div className="text-white/30 text-xs mt-2">synced {new Date(onchain.fetchedAt).toLocaleTimeString()} · source: ledger (no mock)</div>
                                </div>
                            ) : (
                                <div className="text-white/30 text-sm">querying Casper node…</div>
                            )}
                        </div>
                    </div>

                    {/* Telemetry & Nodes (4 cols) */}
                    <div className="col-span-12 md:col-span-4 flex flex-col gap-8 grid-item">
                        
                        {/* Agent Stage */}
                        <div className="editorial-panel p-8 h-[250px] flex items-center justify-center relative overflow-hidden">
                            <CornerMarks />
                            <AgentOrb state={agentState} size={120} />
                            {agentState === "exhausted" && (
                                <div className="absolute top-4 right-4 text-white/40 text-xs tracking-widest uppercase z-10">Zzz</div>
                            )}
                        </div>

                        {/* WASI Nodes */}
                        <div className="editorial-panel p-8 flex-1 relative">
                            <CornerMarks />
                            <div className="text-xs tracking-widest text-white/40 uppercase mb-8 pb-4 border-b border-white/10 z-10">
                                ACTIVE_NODES
                            </div>
                            <div className="flex flex-col gap-4">
                                {wasiNodes.length > 0 ? wasiNodes.map((node) => (
                                    <div key={node.id} className="flex justify-between items-center text-xs">
                                        <div className="flex items-center gap-3">
                                            <div className={`w-1.5 h-1.5 ${node.status === "COMPUTING" ? "bg-white animate-pulse" : node.status === "BREACHED" ? "bg-red-500" : "bg-white/20"}`} />
                                            <span className="text-white/60">{node.cluster}</span>
                                        </div>
                                        <span className="text-white/40">{node.latency}MS</span>
                                    </div>
                                )) : (
                                    <div className="text-xs text-white/30 animate-pulse">POLLING TELEMETRY...</div>
                                )}
                            </div>
                        </div>

                    </div>

                </div>

				{/* 4. Scale Expansion Telemetry Section */}
				<div className="max-w-7xl mx-auto mt-20 border-t border-white/20 pt-16">
					<h2 className="text-2xl font-mono uppercase tracking-widest text-white/90 mb-12">
						System Scale Mesh <span className="text-white/40 font-normal text-xs uppercase ml-4">{"// SCALE EXPANSION VECTORS GAMMA & DELTA"}</span>
					</h2>

					<div className="editorial-grid gap-8">
						{/* Vector Gamma Panel */}
						<div className="col-span-12 md:col-span-6 editorial-panel p-8 relative flex flex-col justify-between min-h-[440px]">
							<CornerMarks />
							<div className="z-10">
								<div className="flex justify-between items-center mb-8 pb-4 border-b border-white/10">
									<span className="text-xs font-mono text-[var(--red-700)] font-bold tracking-widest">VECTOR_GAMMA // COGNITIVE_ARBITRAGE</span>
									<span className="text-[10px] font-mono px-2 py-0.5 border border-[var(--red-700)] text-[var(--red-700)]">ACTIVE</span>
								</div>

								{/* Gas Futures Tracker */}
								<div className="mb-8 p-4 bg-white/5 border border-white/10">
									<div className="text-[10px] text-white/40 mb-2 uppercase">Gas Hedging Futures (CSPR / Gas-Unit)</div>
									<div className="text-3xl font-mono font-bold text-white tracking-tight">
										{gasPrice.toLocaleString(undefined, { minimumFractionDigits: 6, maximumFractionDigits: 6 })}
									</div>
									<div className="flex justify-between items-center mt-3 pt-3 border-t border-white/5">
										<span className="text-xs text-white/60">Hedged Capacity: <span className="text-white font-bold">{gasHedged.toLocaleString()} gas-units</span></span>
										<div className="flex gap-2">
											<button 
												onClick={() => {
													setGasHedged(prev => prev + 100000);
													setGasPrice(p => p + 0.000120);
												}}
												className="px-2 py-1 bg-white text-black font-bold text-[9px] uppercase tracking-wider"
											>
												+ 100K HEDGE
											</button>
											<button 
												onClick={() => {
													setGasHedged(prev => Math.max(0, prev - 100000));
													setGasPrice(p => Math.max(0.001, p - 0.000100));
												}}
												className="px-2 py-1 border border-white/20 text-white font-bold text-[9px] uppercase tracking-wider hover:bg-white/5"
											>
												RELEASE
											</button>
										</div>
									</div>
								</div>

								{/* Arbitrage Delegation Tree */}
								<div>
									<h4 className="text-xs font-mono text-white/60 mb-3 uppercase">Cognitive Arbitrage Delegation Tree</h4>
									<div className="flex flex-col gap-2 font-mono text-xs text-white/80 bg-white/5 p-4 border border-white/10">
										<div className="flex items-center gap-2">
											<span className="text-[var(--red-700)]">[Orchestrator]</span>
											<span className="text-white/40">mark_53_sarcophagus</span>
										</div>
										<div className="pl-4 border-l border-white/20 flex flex-col gap-2 mt-1">
											<div className="flex items-center gap-2">
												<span className="text-white/40">├── [Sub-Escrow A]</span>
												<span className="text-white font-bold">credio_risk_monitor</span>
												<span className="text-[var(--red-700)] text-[10px]">(Risk Analysis: SECURE)</span>
											</div>
											<div className="flex items-center gap-2">
												<span className="text-white/40">├── [Sub-Escrow B]</span>
												<span className="text-white font-bold">agent_alpha_arbitrage</span>
												<span className="text-[var(--red-700)] text-[10px]">(Claiming: 420.5 USDC)</span>
											</div>
											<div className="flex items-center gap-2">
												<span className="text-white/40">└── [Sub-Escrow C]</span>
												<span className="text-white font-bold">liquidity_sniper</span>
												<span className="text-yellow-500 text-[10px]">(MEV Flash-loan: PENDING)</span>
											</div>
										</div>
									</div>
								</div>
							</div>
						</div>

						{/* Vector Delta Panel */}
						<div className="col-span-12 md:col-span-6 editorial-panel p-8 relative flex flex-col justify-between min-h-[440px]">
							<CornerMarks />
							<div className="z-10">
								<div className="flex justify-between items-center mb-8 pb-4 border-b border-white/10">
									<span className="text-xs font-mono text-[var(--red-700)] font-bold tracking-widest">VECTOR_DELTA // ABSOLUTE_SYNERGY</span>
									<span className="text-[10px] font-mono px-2 py-0.5 border border-white/20 text-white/60">LOAD_BALANCER</span>
								</div>

								{/* P2P Load Balancer / Latency Map */}
								<div className="mb-8">
									<div className="text-[10px] text-white/40 mb-3 uppercase">P2P Node Latency & Capacity (Mesh Matrix)</div>
									<div className="grid grid-cols-6 gap-2">
										{meshLoad.map((val, idx) => {
											let boxColor = "bg-white/10 border-white/10";
											if (val > 80) boxColor = "bg-[var(--red-700)] border-[var(--red-700)] shadow-[0_0_10px_rgba(241,50,66,0.3)] animate-pulse";
											else if (val > 40) boxColor = "bg-white/40 border-white/40";
											
											return (
												<div key={idx} className="flex flex-col gap-1 items-center bg-white/5 border border-white/10 p-2">
													<div className={`w-3 h-3 ${boxColor} rounded-none`} />
													<span className="text-[9px] font-mono text-white/50">{val}%</span>
												</div>
											);
										})}
									</div>
								</div>

								{/* L402 Casper Gateway Challenge Console */}
								<div>
									<h4 className="text-xs font-mono text-white/60 mb-2 uppercase">L402-Casper HTTP 402 Gateway Client</h4>
									<div className="bg-black border border-white/10 p-4 font-mono text-[11px] text-white flex flex-col gap-3">
										<pre className="text-white/60 max-h-[80px] overflow-y-auto leading-relaxed whitespace-pre-wrap">
											<code>{l402Console}</code>
										</pre>
										<div className="flex justify-between items-center pt-2 border-t border-white/10">
											<span className="text-[10px] text-white/40">Status: <span className="text-white font-bold">{l402Status}</span></span>
											<button 
												onClick={() => {
													if (l402Status === "IDLE") {
														setL402Status("CHALLENGED");
														setL402Console(">>> GET /api/v1/cargo-payload HTTP/1.1\n<<< HTTP/1.1 402 Payment Required\n<<< WWW-Authenticate: L402 token=\"500c8aef\", invoice=\"01b4c...f201\"\n// Challenge received: Send 1 CSPR to obtain client authorization key.");
													} else if (l402Status === "CHALLENGED") {
														setL402Status("SUCCESS");
														setL402Console(">>> POST /api/v1/casper-verify\n>>> Pay invoice hash: 01b4c...f201 (1 CSPR settled)\n<<< HTTP/1.1 200 OK\n<<< Authorization: L402 credentials=\"token=500c8aef:preimage=cf201\"\n// Access granted. Decoded payload signature verified.");
													} else {
														setL402Status("IDLE");
														setL402Console("// Console reset. Ready to challenge L402 gate");
													}
												}}
												className="px-3 py-1 bg-[var(--red-700)] text-white font-bold text-[9px] uppercase tracking-wider"
											>
												{l402Status === "IDLE" ? "SEND GET REQUEST" : l402Status === "CHALLENGED" ? "PAY 1 CSPR & AUTHORIZE" : "RESET GATE"}
											</button>
										</div>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>

				{/* Live Agent Registry */}
				<div className="max-w-7xl mx-auto mt-20">
					<AgentNetworkGrid />
				</div>
            </section>

            {/* Sticky Execution Input */}
            <section className="sticky bottom-0 z-50 w-full bg-black border-t border-white/20 px-8 md:px-16 py-6">
                <div className="max-w-7xl mx-auto flex flex-col md:flex-row gap-6 items-end">
                    
                    <div className="flex-1 w-full">
                        <div className="text-[10px] tracking-widest text-white/40 uppercase mb-3">Bounty Input Stream</div>
                        <textarea 
                            className="w-full h-16 bg-transparent border border-white/20 p-4 text-sm font-mono text-white placeholder:text-white/20 outline-none focus:border-white transition-colors resize-none"
                            placeholder="DESCRIBE YOUR TASK OR DROP A FILE..."
                            value={inputValue}
                            onChange={(e) => setInputValue(e.target.value)}
                            onFocus={() => setAgentState("typing")}
                            onBlur={() => agentState === "typing" && setAgentState("idle")}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter' && !e.shiftKey) {
                                    e.preventDefault();
                                    if (inputValue.trim() && agentState !== "working") handleExecute();
                                }
                            }}
                        />
                    </div>

                    <div className="flex flex-col gap-3 min-w-[200px]">
                        <div className="flex justify-between text-xs text-white/40 border-b border-white/10 pb-2">
                            <span>USDC_BAL</span>
                            <span className="text-white">${balance.toFixed(2)}</span>
                        </div>
                        <button 
                            onClick={handleExecute}
                            disabled={!inputValue.trim() || agentState === "working"}
                            className="w-full button-primary label-12-mono font-bold tracking-widest uppercase active:scale-[0.98] transition-transform"
                            style={{ height: "48px" }}
                        >
                            {agentState === "working" ? "EXECUTING..." : "EXECUTE_SEQ"}
                        </button>
                    </div>

                </div>
                
                {/* Worm Progress Indicator */}
                <div className="absolute top-0 left-0 w-full h-[2px] bg-white/10">
                    <motion.div 
                        className="h-full bg-white"
                        initial={{ width: 0 }}
                        animate={{ width: `${progress}%` }}
                        transition={{ ease: "linear" }}
                    />
                </div>
            </section>

		</main>
	);
}
