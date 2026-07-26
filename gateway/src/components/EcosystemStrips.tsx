"use client";
import { useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// The substance slot: what the mesh IS and what it stands on. Angled Off-White / Adidas strips
// that drift as you scroll — the product's own guarantees, not hackathon logistics or a logo wall.
// Every strip is CLICKABLE and points at its REAL source of truth — preferring the NATIVE proof:
// live Casper-testnet transactions / contract packages on cspr.live, the native x402 tooling, the
// open repo. All hashes/URLs verified against DEPLOYMENTS.md — nothing invented.
const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const EXP = "https://testnet.cspr.live";
const PKG_ESCROW = "a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d";
const PKG_ORACLE = "16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2";
const ACCOUNT = "013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35";
const TX_ESCROW_PAYOUT = "1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14"; // release_bounty
const TX_AGENT_RULING = "c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89"; // LLM APPROVE -> payout

const STRIPS: { tag: string; label: string; href: string; accent?: boolean }[] = [
	{ tag: "THE MESH", label: "Machines that hire, pay & settle each other", accent: true, href: `${REPO}/blob/main/README.md` },
	{ tag: "SETTLEMENT", label: "Casper · Rust / WASM", href: `${EXP}/contract-package/${PKG_ESCROW}` },
	{ tag: "ESCROW", label: "Funds held in code, released on proof", href: `${EXP}/transaction/${TX_ESCROW_PAYOUT}` },
	{ tag: "ORACLE", label: "Truth verified on-chain", href: `${EXP}/contract-package/${PKG_ORACLE}` },
	{ tag: "TRIBUNAL", label: "Disputes ruled by protocol, not people", href: `${EXP}/transaction/${TX_AGENT_RULING}` },
	{ tag: "PAYMENTS", label: "x402 native rails", href: "https://github.com/odradev/casper-x402-poc" },
	{ tag: "STATUS", label: "Live on Casper testnet", href: `${EXP}/account/${ACCOUNT}` },
	{ tag: "SOURCE", label: "Open · public · verifiable", href: REPO },
	{ tag: "THE MANDATE", label: "Agentic AI · DeFi · RWA — on Casper", accent: true, href: `${REPO}/blob/main/VISION.md` },
];

function Strip({ s, i, progress }: { s: (typeof STRIPS)[number]; i: number; progress: MotionValue<number> }) {
	const dir = i % 2 === 0 ? 1 : -1;
	// each drifts in from its side and settles — "reacting to scroll"
	const x = useTransform(progress, [0, 0.6], [dir * (120 + i * 14), 0]);
	const opacity = useTransform(progress, [0, 0.35], [0, 1]);
	const indent = (i % 3) * 3.2; // staggered left offset -> diagonal cascade

	return (
		<motion.a
			href={s.href}
			target="_blank"
			rel="noopener noreferrer"
			style={{ x, opacity, skewX: -11, marginLeft: `${indent}vw` }}
			// premium hover: the strip lifts + scales a touch, border/glow warms to red.
			whileHover={{ scale: 1.035 }}
			transition={{ type: "spring", stiffness: 300, damping: 20 }}
			className={`group w-fit block cursor-pointer border-l-2 ${s.accent ? "border-[var(--red-700)]" : "border-[#2a2024]"} bg-[#0a0608]/85 backdrop-blur-[2px] pl-5 pr-8 py-3.5 flex items-center will-change-transform transition-colors duration-300 hover:bg-[#140a0f]/92 hover:border-[var(--red-700)] hover:shadow-[0_0_28px_-6px_rgba(224,53,41,0.45)]`}
		>
			<div style={{ transform: "skewX(11deg)" }} className="flex items-baseline gap-4 whitespace-nowrap">
				<span className={`label-13-mono uppercase tracking-[0.16em] transition-colors duration-300 ${s.accent ? "text-[var(--red-700)]" : "text-[var(--gray-800)]/70 group-hover:text-[var(--red-700)]/90"}`}>
					[ {s.tag} ]
				</span>
				<span
					className="uppercase tracking-tight text-white/85 transition-colors duration-300 group-hover:text-white"
					style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 400, fontSize: "clamp(16px, 1.5vw, 30px)" }}
				>
					{s.label}
				</span>
				{/* proof indicator — slides in on hover, signalling "this opens the real source" */}
				<span className="label-13-mono text-white/0 -translate-x-1.5 group-hover:text-white/55 group-hover:translate-x-0 transition-all duration-300 self-center">
					↗
				</span>
			</div>
		</motion.a>
	);
}

export function EcosystemStrips() {
	const ref = useRef<HTMLDivElement>(null);
	const { scrollYProgress } = useScroll({ target: ref, offset: ["start end", "center center"] });

	return (
		<div ref={ref} className="w-full overflow-hidden">
			<div className="mb-[7vh] flex items-baseline gap-4 flex-wrap">
				<span className="label-13-mono text-[var(--red-700)] uppercase tracking-[0.2em]">{"///"} forged in the open</span>
				<span className="label-13-mono text-[var(--gray-800)]/60 uppercase tracking-[0.14em]">— the mesh, its guarantees, its settlement · every line opens its proof</span>
			</div>

			<div className="flex flex-col gap-3.5">
				{STRIPS.map((s, i) => (
					<Strip key={i} s={s} i={i} progress={scrollYProgress} />
				))}
			</div>

			<p className="mt-[8vh] max-w-[54ch] label-13-mono text-[var(--gray-800)]/70 uppercase" style={{ lineHeight: 1.8 }}>
				A global arena for the next generation of on-chain agents. We built Triarchy here — in
				the open, source public, settling live on Casper testnet.
			</p>
		</div>
	);
}
