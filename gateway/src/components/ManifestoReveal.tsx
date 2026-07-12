"use client";
import { useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// The thesis — the bridge from the hero's emotion + the pillar cards to conviction.
// Not "here are our technologies", but WHY this has to exist. Confident, restrained.
// Two accented keywords (accountability / Casper) light up red as you scroll it in.
const WORDS: { t: string; accent?: boolean }[] = [
	{ t: "Everyone" }, { t: "is" }, { t: "racing" }, { t: "to" }, { t: "make" },
	{ t: "machines" }, { t: "more" }, { t: "autonomous." },
	{ t: "We" }, { t: "build" }, { t: "the" }, { t: "part" }, { t: "that" }, { t: "makes" },
	{ t: "autonomy" }, { t: "safe" }, { t: "to" }, { t: "trust" }, { t: "—" },
	{ t: "accountability,", accent: true }, { t: "written" }, { t: "in" }, { t: "code," },
	{ t: "settled" }, { t: "on" }, { t: "Casper.", accent: true },
];

function Word({ w, progress, start, end }: { w: { t: string; accent?: boolean }; progress: MotionValue<number>; start: number; end: number }) {
	const opacity = useTransform(progress, [start, end], [0.12, 1]);
	return (
		<motion.span style={{ opacity }} className={w.accent ? "text-[var(--red-700)]" : "text-white"}>
			{w.t}{" "}
		</motion.span>
	);
}

export function ManifestoReveal() {
	const ref = useRef<HTMLDivElement>(null);
	const { scrollYProgress } = useScroll({ target: ref, offset: ["start 0.82", "start 0.28"] });
	const n = WORDS.length;

	return (
		<div ref={ref} className="w-full max-w-[74vw] max-lg:max-w-none">
			<p
				className="uppercase-none tracking-tight leading-[1.18]"
				style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, fontSize: "clamp(28px, 3.65vw, 78px)" }}
			>
				{WORDS.map((w, i) => {
					const start = (i / n) * 0.82;
					return <Word key={i} w={w} progress={scrollYProgress} start={start} end={start + 0.2} />;
				})}
			</p>

			<div className="mt-[9vh] max-w-[52ch]">
				<p className="label-13-mono text-[var(--gray-800)] uppercase" style={{ lineHeight: 1.75 }}>
					Escrow that holds. An oracle that checks. A tribunal that rules. Every verdict final,
					on-chain — not a promise, a protocol.
				</p>
				<p className="label-13-mono text-[var(--red-700)] uppercase mt-5 tracking-[0.18em]">
					Built on Casper · Rust/WASM · real settlement.
				</p>
			</div>
		</div>
	);
}
