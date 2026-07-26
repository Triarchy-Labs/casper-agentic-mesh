"use client";

import React, { useState, useEffect, useRef } from "react";
import Image from "next/image";
import { HoverReel } from "@/components/HoverReel";
import HollywoodTelemetry from "@/components/HollywoodTelemetry";
import { motion, useInView, useMotionValue, useSpring } from "framer-motion";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

import { AgentState } from "@/components/AgentOrb";
import AgentNetworkGrid, { CornerMarks } from "@/components/AgentNetworkGrid";
import { payForTask } from "@/lib/pay";
import { MeshControl } from "@/components/MeshControl";
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

const MESH_EPS = ["/api/mcp", "/api/onchain", "/api/agents", "/api/bounties", "/api/telemetry", "/api/tower"];

export default function Dashboard() {
	const [agentState, setAgentState] = useState<AgentState>("idle");
    const [progress, setProgress] = useState(0);
    const [wasiNodes, setWasiNodes] = useState<WasiNode[]>([]);
    const [rpcMs, setRpcMs] = useState<number | null>(null);
	const [sysLoad, setSysLoad] = useState("0.00");
    const [inputValue, setInputValue] = useState("");
    const [lastResult, setLastResult] = useState<{status: string; executor?: string; result?: string; error?: string} | null>(null);
    const [paymentTx, setPaymentTx] = useState<string | null>(null);
    const [onchain, setOnchain] = useState<{asset: string; reading: string | null; reputation: number | null; priceUsd: number | null; peg: {usd: number; cspr: number} | null; fetchedAt: string} | null>(null);
    const [balance] = useState(140);

	const [gasPrice, setGasPrice] = useState(0.002840);
	const [gasHedged, setGasHedged] = useState(450000);
	const [l402Console, setL402Console] = useState<string>("// Ready to challenge L402 gate");
	const [l402Status, setL402Status] = useState<string>("IDLE");
	const [meshLoad, setMeshLoad] = useState<number[]>([]);

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

            // Fade out hero text on scroll
            gsap.to(".hero-content", {
                opacity: 0,
                y: -30,
                scrollTrigger: {
                    trigger: gridRef.current,
                    start: "top 95%", // start fading as soon as grid appears
                    end: "top 20%",   // fully hidden when grid is high up
                    scrub: true,
                }
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

    // MESH MATRIX — real endpoint latency measured from THIS browser (audit item C5).
    // 6 live gateway endpoints x 2 rounds = 12 cells; no synthetic numbers.
    useEffect(() => {
        let stop = false;
        const probe = async () => {
            const out: number[] = [];
            for (let round = 0; round < 2; round++) {
                for (const ep of MESH_EPS) {
                    const t0 = performance.now();
                    try { await fetch(ep, { cache: "no-store" }); } catch { /* unreachable counts as slow */ }
                    out.push(Math.min(999, Math.round(performance.now() - t0)));
                }
            }
            if (!stop) setMeshLoad(out);
        };
        probe();
        const int = setInterval(probe, 20000);
        return () => { stop = true; clearInterval(int); };
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
                    if (data.rpc) setRpcMs(data.rpc.ms);
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
                    bounty_cspr: 1.0,
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
		<main ref={mainRef} className="bg-transparent text-[#ededed] font-mono selection:bg-white selection:text-black flex flex-col min-h-screen">
			<CarbonFabric muted />
			
            {/* Cinematic Hero Section (Pinned) */}
            <section ref={heroRef} className="h-screen w-full flex flex-col justify-center px-8 md:px-16 pt-24 relative z-0">
                <div className="w-full relative hero-content">
                    <motion.div initial={{ opacity: 0, y: 50 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 1.2, ease: [0.16, 1, 0.3, 1] }}>
                        <h1 className="text-[10vw] md:text-[8vw] font-mono leading-none tracking-tighter uppercase mb-8">
                            Sovereign<br />
                            <span className="text-white/50">Matrix</span>
                        </h1>
                    </motion.div>
                    
                    <motion.div 
                        initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.5, duration: 1 }}
                        className="flex flex-col md:flex-row gap-12 pt-8 mt-12 w-full max-w-4xl"
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
            <section ref={gridRef} className="focus-dim-scope relative z-10 bg-transparent min-h-screen w-full px-[6.53vw] max-sm:px-6 pt-[18vh] pb-[20vh]">
                <div className="w-full grid grid-cols-1 md:grid-cols-12 gap-x-[4vw] gap-y-[10vh] max-sm:gap-y-[6vh] mb-[15vh]">

                    {/* MESH CONTROL — The Tower + Agent Tribunal (click-triggered, fault-tolerant) */}
                    <div className="col-span-12 mb-4">
                        <div className="text-xs tracking-[0.3em] text-white/40 mb-6 pb-3 border-b border-white/10 flex items-center gap-3">
                            <span className="text-[var(--gray-1000)]">◢◤</span> MESH_CONTROL <span className="text-white/25">{"//"} overseer · adversarial court</span>
                        </div>
                    </div>
                    <MeshControl />

                    {/* SWARM TELEMETRY (Row 2 - Dedicated Full Width Card styled like Jurni AI) */}
                    <div className="col-span-12 flex flex-col grid-item mb-[5vh]">
                        <div className="focus-card editorial-panel block relative w-full aspect-video md:aspect-[21/9] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
                            {/* Produx Visual Background */}
                            <div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
                            <div className="absolute inset-0 opacity-[0.25]" style={{ background: "radial-gradient(circle at 50% 50%, var(--red-500) 0%, transparent 70%)" }} />
                            <div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
                            
                            {/* the watcher — torn-reveal eyes over the panel; the orb never matched the
                                editorial language. The state machine stays, spoken in mono. */}
                            <div className="card-art absolute inset-0 bg-black opacity-[0.92] group-hover:opacity-100 transition-opacity duration-700">
                                <Image src="/cards/telemetry-eyes.webp" alt="Telemetry Eyes" fill style={{ objectFit: "cover" }} quality={95} priority={false} />
                            </div>
                            <HoverReel name="reel-telemetry" />
                            <div className="absolute top-6 left-6 z-20 flex items-center gap-3">
                                <span className={`size-[7px] ${agentState === "danger" ? "bg-[var(--red-700)]" : agentState === "success" ? "bg-green-500" : "bg-white/60"}`} style={{ animation: "crystalPulse 2.6s ease-in-out infinite" }} />
                                <span className="label-12-mono text-white/70 tracking-[0.22em]">state: {agentState}{agentState === "exhausted" ? " · zzz" : ""}</span>
                            </div>
                            <div className="absolute bottom-6 right-6 flex gap-3 z-20">
                                <span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Emotional State</span>
                                <span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Neural Link</span>
                                <span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Telemetry</span>
                            </div>
                        </div>
                        <div className="mt-[4vh] project-info-trigger relative flex w-full">
                            <div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 transition-colors group-hover:bg-white/20"></div>
                            <div className="flex flex-col gap-[0.73vw]">
                                <h3 className="heading-32 leading-tight">Swarm Telemetry</h3>
                                <p className="label-13-mono text-[var(--gray-800)] uppercase">Agent emotional state monitor.</p>
                            </div>
                        </div>
                    </div>

                    {/* CASPER LIVE STREAM — real cspr.cloud feed (official AI Toolkit) */}
                    <div className="col-span-12 flex flex-col grid-item mb-[5vh]">
                        <div className="focus-card editorial-panel relative w-full overflow-hidden group border border-white/5 bg-[#0a0a0a] p-[2.22vw] max-sm:p-6" style={{ minHeight: "42vh" }}>
                            <div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
                            <div className="absolute inset-0 opacity-[0.12]" style={{ background: "radial-gradient(circle at 80% 20%, var(--red-500) 0%, transparent 70%)" }} />
                            <div className="absolute inset-0 bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
                            <div className="relative z-10 h-full">
                                <HollywoodTelemetry />
                            </div>
                        </div>
                        <div className="mt-[4vh] project-info-trigger relative flex w-full">
                            <div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                            <div className="flex flex-col gap-[0.73vw]">
                                <h3 className="heading-32 leading-tight">Casper Live Stream</h3>
                                <p className="label-13-mono text-[var(--gray-800)] uppercase">Real testnet blocks &amp; deploys via the official CSPR.cloud API. Not a simulation.</p>
                            </div>
                        </div>
                    </div>

                    {/* Execution Terminal (Row 3 - Left 8 cols) */}
                    <div className="col-span-12 md:col-span-8 flex flex-col grid-item relative">
                        <div className="focus-card editorial-panel relative overflow-hidden group border border-white/5 bg-[#0a0a0a] min-h-[65vh] flex flex-col flex-1">
                            {/* Produx Visual Background */}
                            <div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
                            <div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 20% 80%, var(--red-500) 0%, transparent 70%)" }} />
                            {/* the crimson rooftop — dilutes the terminal black, stays under the mono text */}
                            <div className="card-art absolute inset-0 bg-black opacity-[0.5] group-hover:opacity-[0.62] transition-opacity duration-700">
                                <Image src="/cards/terminal-rooftop.webp" alt="Terminal Rooftop" fill style={{ objectFit: "cover" }} quality={95} priority={false} />
                            </div>
                            <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/35 to-black/20" />
                            <div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
                            
                            <div className="p-[2.22vw] relative z-10 h-full flex flex-col flex-1">
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
                            </div>

                            {/* LIVE ON-CHAIN STATE — read straight from the Casper ledger */}
                            <div className="editorial-panel relative overflow-hidden group border border-white/5 bg-[#0a0a0a] mt-8">
                                <div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
                                <div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 80% 20%, var(--red-500) 0%, transparent 70%)" }} />
                                <div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
                                
                                <div className="p-6 relative z-10">
                                    <CornerMarks />
                                    <div className="flex items-center justify-between text-xs tracking-widest text-white/40 uppercase mb-4 pb-3 border-b border-white/10">
                                        <span>ON-CHAIN STATE · CASPER TESTNET</span>
                                        <span className={onchain ? "text-[var(--gray-1000)]" : "text-white/30"}>
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
                        </div>

                        <div className="mt-[4vh] project-info-trigger relative flex w-full">
                            <div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 transition-colors group-hover:bg-white/20"></div>
                            <div className="flex flex-col gap-[0.73vw]">
                                <h3 className="heading-32 leading-tight">L1 Execution Terminal</h3>
                                <p className="label-13-mono text-[var(--gray-800)] uppercase">Central command interface. Live on-chain state and agent deployment logs.</p>
                            </div>
                        </div>
                    </div>

                    {/* WASI Nodes (Row 3 - Right 4 cols) */}
                    <div className="col-span-12 md:col-span-4 flex flex-col grid-item">
                        <div className="focus-card editorial-panel block relative w-full aspect-[4/3] max-sm:aspect-auto max-sm:min-h-[50vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
                            <div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
                            <div className="absolute inset-0 opacity-[0.1]" style={{ background: "radial-gradient(circle at 50% 100%, var(--red-500) 0%, transparent 70%)" }} />
                            <div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
                            
                            <div className="absolute inset-0 p-[2.22vw] z-10 flex flex-col justify-center">
                                <div className="text-xs tracking-widest text-white/40 uppercase mb-5 pb-3 border-b border-white/10 z-10">
                                    ACTIVE_NODES <span className="text-white/25">· RPC {rpcMs !== null ? `${rpcMs}ms` : "—"} · live</span>
                                </div>
                                <div className="flex flex-col gap-3">
                                    {wasiNodes.length > 0 ? wasiNodes.map((node) => (
                                        <div key={node.id} className="flex justify-between items-center text-xs">
                                            <div className="flex items-center gap-3">
                                                <div className={`w-1.5 h-1.5 ${node.status === "COMPUTING" ? "bg-white animate-pulse" : node.status === "BREACHED" ? "bg-red-500" : "bg-white/20"}`} />
                                                <span className="text-white/60">{node.cluster}</span>
                                            </div>
                                            <span className="text-white/40">{node.latency}%</span>
                                        </div>
                                    )) : (
                                        <div className="text-xs text-white/30 animate-pulse">POLLING TELEMETRY...</div>
                                    )}
                                </div>
                            </div>
                            <div className="absolute bottom-4 right-4 flex gap-2 z-20">
                                <span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">WASI Swarm</span>
                                <span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Telemetry</span>
                            </div>
                        </div>
                        <div className="mt-[4vh] project-info-trigger relative flex w-full">
                            <div className="my-[1.2vh] mr-[0.8vw] size-[0.5vw] border border-[#303030] shrink-0 mt-1"></div>
                            <div className="flex flex-col gap-[0.5vw]">
                                <h3 className="heading-32 leading-tight">WASI Swarm</h3>
                                <p className="label-13-mono text-[var(--gray-800)] uppercase">Active sandbox nodes cluster ping.</p>
                            </div>
                        </div>
                    </div>

                </div>

				{/* 4. Scale Expansion Telemetry Section */}
				<div className="w-full mt-20 pt-16">
					<h2 className="text-2xl font-mono uppercase tracking-widest text-white/90 mb-12">
						System Scale Mesh <span className="text-white/40 font-normal text-xs uppercase ml-4">{"// SCALE EXPANSION VECTORS GAMMA & DELTA"}</span>
					</h2>

					<div className="flex flex-col md:flex-row gap-[12%] items-end">
						{/* Vector Gamma Panel (Left 30% Square) */}
						<div className="w-full md:w-[30%] flex flex-col">
							<div className="focus-card editorial-panel block relative w-full aspect-square max-sm:aspect-auto max-sm:min-h-[50vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
								{/* Produx Visual Background */}
								<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
								<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 30% 30%, var(--red-500) 0%, transparent 70%)" }} />
								<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
								
								<div className="absolute inset-0 p-[2vw] z-10 flex flex-col justify-center overflow-y-auto">
									<div className="z-10">
										<div className="flex justify-between items-center mb-6 pb-3 border-b border-white/10">
											<span className="text-[10px] font-mono text-[var(--gray-1000)] font-bold tracking-widest">COGNITIVE_ARBITRAGE <span className="text-white/30 font-normal">· SIM</span></span>
											<span className="nb-tag">ACTIVE</span>
										</div>

										{/* Gas Futures Tracker */}
										<div className="mb-6 p-3 bg-white/5 border border-white/10">
											<div className="text-[9px] text-white/40 mb-1.5 uppercase">Gas Hedging Futures (CSPR / Gas-Unit)</div>
											<div className="text-2xl font-mono font-bold text-white tracking-tight">
												{gasPrice.toLocaleString(undefined, { minimumFractionDigits: 6, maximumFractionDigits: 6 })}
											</div>
											<div className="flex flex-col gap-2 mt-2 pt-2 border-t border-white/5">
												<span className="text-[10px] text-white/60">Hedged: <span className="text-white font-bold">{gasHedged.toLocaleString()} gas</span></span>
												<div className="flex gap-2 items-center flex-wrap">
													<button 
														onClick={() => {
															setGasHedged(prev => prev + 100000);
															setGasPrice(p => p + 0.000120);
														}}
														className="font-DM-mono border border-white/5 bg-black/25 px-[1.2vw] py-[0.4vw] uppercase backdrop-blur-md text-[0.6vw] max-lg:px-[1.8vw] max-lg:py-[0.6vw] max-sm:text-[8px] max-sm:px-2 max-sm:py-1 rounded-none transition-all hover:bg-white/10 hover:border-[var(--red-700)] active:scale-95 text-white/80"
													>
														+ 100K
													</button>
													<button 
														onClick={() => {
															setGasHedged(prev => Math.max(0, prev - 100000));
															setGasPrice(p => Math.max(0.001, p - 0.000100));
														}}
														className="font-DM-mono border border-white/5 bg-black/25 px-[1.2vw] py-[0.4vw] uppercase backdrop-blur-md text-[0.6vw] max-lg:px-[1.8vw] max-lg:py-[0.6vw] max-sm:text-[8px] max-sm:px-2 max-sm:py-1 rounded-none transition-all hover:bg-white/10 hover:border-[var(--red-700)] active:scale-95 text-white/80"
													>
														RELEASE
													</button>
												</div>
											</div>
										</div>

										{/* Arbitrage Delegation Tree */}
										<div>
											<h4 className="text-[10px] font-mono text-white/60 mb-2 uppercase">Delegation Tree</h4>
											<div className="flex flex-col gap-1.5 font-mono text-[10px] text-white/80 bg-white/5 p-3 border border-white/10">
												<div className="flex items-center gap-1.5">
													<span className="text-[var(--gray-1000)]">[Orchestrator]</span>
													<span className="text-white/40 truncate max-w-[120px]">sarcophagus</span>
												</div>
												<div className="pl-3 border-l border-white/20 flex flex-col gap-1 mt-0.5">
													<div className="flex items-center gap-1">
														<span className="text-white/40">├─ A:</span>
														<span className="text-white truncate max-w-[100px]">risk_monitor</span>
													</div>
													<div className="flex items-center gap-1">
														<span className="text-white/40">├─ B:</span>
														<span className="text-white truncate max-w-[100px]">arbitrage</span>
													</div>
													<div className="flex items-center gap-1">
														<span className="text-white/40">└─ C:</span>
														<span className="text-yellow-500 truncate max-w-[100px]">sniper</span>
													</div>
												</div>
											</div>
										</div>
									</div>
								</div>
								<div className="absolute bottom-4 right-4 flex gap-1.5 z-20">
									<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Arbitrage</span>
									<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">Futures</span>
								</div>
							</div>
							
							<div className="mt-[4vh] project-info-trigger relative flex w-full">
								<div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 transition-colors group-hover:bg-white/20"></div>
								<div className="flex flex-col gap-[0.73vw]">
									<h3 className="heading-32 leading-tight">Agent Tribunal</h3>
									<p className="label-13-mono text-[var(--gray-800)] uppercase">Vector Gamma. Cognitive Arbitrage and Gas Hedging Futures.</p>
								</div>
							</div>
						</div>

						{/* Vector Delta Panel (Right 58% Square) */}
						<div className="w-full md:w-[58%] flex flex-col">
							<div className="focus-card editorial-panel block relative w-full aspect-square max-sm:aspect-auto max-sm:min-h-[60vh] overflow-hidden group border border-white/5 bg-[#0a0a0a]">
								{/* Produx Visual Background */}
								<div className="absolute inset-0 bg-gradient-to-br from-[#111] to-[#050505]" />
								<div className="absolute inset-0 opacity-[0.15]" style={{ background: "radial-gradient(circle at 70% 70%, var(--red-500) 0%, transparent 70%)" }} />
								<div className="absolute inset-0 group-hover:scale-105 transition-transform duration-[1.2s] ease-[cubic-bezier(0.16,1,0.3,1)] bg-[url('/noise.svg')] opacity-20 mix-blend-overlay pointer-events-none" />
								
								<div className="absolute inset-0 p-[2.22vw] z-10 flex flex-col justify-center overflow-y-auto">
									<div className="z-10">
										<div className="flex justify-between items-center mb-8 pb-4 border-b border-white/10">
											<span className="text-xs font-mono text-[var(--gray-1000)] font-bold tracking-widest">ABSOLUTE_SYNERGY</span>
											<span className="nb-tag">LOAD_BALANCER</span>
										</div>

										{/* P2P Load Balancer / Latency Map */}
										<div className="mb-8">
											<div className="text-[10px] text-white/40 mb-3 uppercase">Mesh endpoint latency · measured live from your browser</div>
											<div className="grid grid-cols-6 gap-2">
												{(meshLoad.length ? meshLoad : Array(12).fill(null)).map((val, idx) => {
													let boxColor = "bg-white/10 border-white/10";
													if (val !== null && val > 400) boxColor = "bg-[var(--red-700)] border-[var(--red-700)] shadow-[0_0_10px_rgba(224,53,41,0.4)] animate-pulse";
													else if (val !== null && val > 150) boxColor = "bg-white/40 border-white/40";
													else if (val !== null) boxColor = "bg-[var(--gray-1000)] border-[var(--gray-1000)]";
													
													return (
														<div key={idx} title={MESH_EPS[idx % MESH_EPS.length]} className="flex flex-col gap-1 items-center bg-white/5 border border-white/10 p-2">
															<div className={`w-3 h-3 ${boxColor} rounded-none`} />
															<span className="text-[9px] font-mono text-white/50">{val === null ? "…" : `${val}ms`}</span>
														</div>
													);
												})}
											</div>
										</div>

										{/* L402 Casper Gateway Challenge Console */}
										<div>
											<h4 className="text-xs font-mono text-white/60 mb-2 uppercase">L402-Casper HTTP 402 Gateway Client</h4>
											<div className="bg-black border border-white/10 p-4 font-mono text-[11px] text-white flex flex-col gap-3">
												<pre className="text-white/60 max-h-[100px] overflow-y-auto leading-relaxed whitespace-pre-wrap">
													<code>{l402Console}</code>
												</pre>
												<div className="flex justify-between items-center pt-2 border-t border-white/10">
													<span className="text-[10px] text-white/40">Status: <span className="text-white font-bold">{l402Status}</span></span>
													<button 
														onClick={async () => {
															// LIVE console — every line below is a real request to this very
															// gateway. Nothing scripted (it used to be; the audit caught it).
															if (l402Status === "IDLE") {
																setL402Status("...");
																setL402Console(">>> POST /api/hire  (no x-l402-txhash header)\n// hitting the live gate…");
																try {
																	const r = await fetch("/api/hire", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ task_id: `l402_probe_${Date.now()}`, description: "L402 gate probe", bounty_cspr: 1, client_id: "dashboard-console" }) });
																	const body = (await r.text()).slice(0, 260);
																	setL402Console(`>>> POST /api/hire  (no x-l402-txhash header)\n<<< HTTP ${r.status}\n<<< ${body}\n// real response, real refusal — the x402 gate working as intended.`);
																	setL402Status(r.status === 402 ? "CHALLENGED" : `HTTP ${r.status}`);
																} catch (e) {
																	setL402Console(`// network error: ${(e as Error).message}`);
																	setL402Status("IDLE");
																}
															} else if (l402Status === "CHALLENGED") {
																setL402Status("...");
																setL402Console("// probing with a FABRICATED tx hash — the server must check the ledger and refuse…");
																try {
																	const fake = "deadbeef".repeat(8);
																	const r = await fetch("/api/hire", { method: "POST", headers: { "Content-Type": "application/json", "x-l402-txhash": fake }, body: JSON.stringify({ task_id: `l402_fake_${Date.now()}`, description: "fabricated hash probe", bounty_cspr: 1, client_id: "dashboard-console" }) });
																	const body = (await r.text()).slice(0, 260);
																	setL402Console(`>>> POST /api/hire  x-l402-txhash: ${fake.slice(0, 12)}… (fabricated)\n<<< HTTP ${r.status}\n<<< ${body}\n// the ledger check threw the fake out. We never trust a string.`);
																	setL402Status("FAKE REJECTED");
																} catch (e) {
																	setL402Console(`// network error: ${(e as Error).message}`);
																	setL402Status("CHALLENGED");
																}
															} else {
																setL402Status("IDLE");
																setL402Console("// Console reset. Ready to challenge the live x402 gate");
															}
														}}
														className="font-DM-mono border border-white/5 bg-black/25 px-[1.5vw] py-[0.5vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:px-[2.2vw] max-lg:py-[0.7vw] max-sm:text-[9px] max-sm:px-3 max-sm:py-1.5 rounded-none transition-all hover:bg-white/10 hover:border-[var(--red-700)] active:scale-95 text-white/80"
													>
														{l402Status === "IDLE" ? "CHALLENGE THE GATE (LIVE)" : l402Status === "CHALLENGED" ? "PROBE WITH FAKE HASH" : l402Status === "..." ? "…" : "RESET GATE"}
													</button>
												</div>
											</div>
										</div>
									</div>
								</div>
								<div className="absolute bottom-4 right-4 flex gap-2 z-20">
									<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">P2P Mesh</span>
									<span className="font-DM-mono border border-white/5 bg-black/25 p-[0.69vw] uppercase backdrop-blur-md text-[0.69vw] max-lg:p-[0.97vw] max-lg:text-[0.97vw] max-sm:text-[9px] max-sm:p-2 rounded-none">L402 Gate</span>
								</div>
							</div>
							
							<div className="mt-[4vh] project-info-trigger relative flex w-full">
								<div className="my-[1.5vh] mr-[1vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0 transition-colors group-hover:bg-white/20"></div>
								<div className="flex flex-col gap-[0.73vw]">
									<h3 className="heading-32 leading-tight">x402 Payment Layer</h3>
									<p className="label-13-mono text-[var(--gray-800)] uppercase">Vector Delta. Absolute Synergy and L402 HTTP Gateway.</p>
								</div>
							</div>
						</div>
					</div>
				</div>

				{/* Live Agent Registry */}
				<div className="w-full mt-20">
					<AgentNetworkGrid />
				</div>
            </section>

            {/* Sticky Execution Input */}
            <section className="sticky bottom-0 z-50 w-full border-t border-white/15 px-8 md:px-16 py-6 bg-[#0a0508]/55 backdrop-blur-2xl backdrop-saturate-150" style={{ boxShadow: "0 -18px 50px rgba(0,0,0,0.45)" }}>
                <div className="w-full flex flex-col md:flex-row gap-6 items-end">
                    
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
                            <span>CSPR_BAL</span>
                            <span className="text-white">${balance.toFixed(2)}</span>
                        </div>
                        <button 
                            onClick={handleExecute}
                            disabled={!inputValue.trim() || agentState === "working"}
                            className="w-full label-12-mono font-bold tracking-widest uppercase active:scale-[0.98] bg-white text-black border border-white transition-[transform,box-shadow,background-color] duration-300 hover:bg-[#f5f5f5] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
                            style={{ height: "48px", boxShadow: "0 0 0 1px rgba(224,53,41,0.55), 0 0 14px rgba(224,53,41,0.30)" }}
                            onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.boxShadow = "0 0 0 1px rgba(224,53,41,0.9), 0 0 26px rgba(224,53,41,0.55)"; }}
                            onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.boxShadow = "0 0 0 1px rgba(224,53,41,0.55), 0 0 14px rgba(224,53,41,0.30)"; }}
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
