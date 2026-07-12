"use client";
import { useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// The context slot: who this was forged with. Angled Off-White / Adidas strips that
// drift as you scroll — the hackathon ecosystem, not a boring logo wall.
const STRIPS: { tag: string; label: string; accent?: boolean }[] = [
	{ tag: "THE ARENA", label: "Casper Agentic Buildathon 2026", accent: true },
	{ tag: "PRIZE POOL", label: "$150,000 — cash + x402 ecosystem credits" },
	{ tag: "ORGANIZER", label: "Casper Association" },
	{ tag: "PLATFORM", label: "DoraHacks" },
	{ tag: "IRL", label: "Istanbul Blockchain Week" },
	{ tag: "SECURITY", label: "Halborn" },
	{ tag: "DATA", label: "Allium" },
	{ tag: "INFRA", label: "NOWNodes" },
	{ tag: "THE MANDATE", label: "Agentic AI · DeFi · RWA — on Casper", accent: true },
];

function Strip({ s, i, progress }: { s: (typeof STRIPS)[number]; i: number; progress: MotionValue<number> }) {
	const dir = i % 2 === 0 ? 1 : -1;
	// each drifts in from its side and settles — "reacting to scroll"
	const x = useTransform(progress, [0, 0.6], [dir * (120 + i * 14), 0]);
	const opacity = useTransform(progress, [0, 0.35], [0, 1]);
	const indent = (i % 3) * 3.2; // staggered left offset -> diagonal cascade

	return (
		<motion.div
			style={{ x, opacity, skewX: -11, marginLeft: `${indent}vw` }}
			className={`w-fit border-l-2 ${s.accent ? "border-[var(--red-700)]" : "border-[#2a2024]"} bg-[#0a0608]/85 backdrop-blur-[2px] pl-5 pr-8 py-3.5 flex items-center will-change-transform`}
		>
			<div style={{ transform: "skewX(11deg)" }} className="flex items-baseline gap-4 whitespace-nowrap">
				<span className={`label-13-mono uppercase tracking-[0.16em] ${s.accent ? "text-[var(--red-700)]" : "text-[var(--gray-800)]/70"}`}>
					[ {s.tag} ]
				</span>
				<span
					className={`uppercase tracking-tight ${s.accent ? "text-white" : "text-white/85"}`}
					style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 400, fontSize: "clamp(16px, 1.5vw, 30px)" }}
				>
					{s.label}
				</span>
			</div>
		</motion.div>
	);
}

export function EcosystemStrips() {
	const ref = useRef<HTMLDivElement>(null);
	const { scrollYProgress } = useScroll({ target: ref, offset: ["start end", "center center"] });

	return (
		<div ref={ref} className="w-full overflow-hidden">
			<div className="mb-[7vh] flex items-baseline gap-4 flex-wrap">
				<span className="label-13-mono text-[var(--red-700)] uppercase tracking-[0.2em]">/// forged in the open</span>
				<span className="label-13-mono text-[var(--gray-800)]/60 uppercase tracking-[0.14em]">— the ecosystem that put the arena up</span>
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
