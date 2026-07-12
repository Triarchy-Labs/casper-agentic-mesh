"use client";
import { useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// Ported 1:1 from produx's journal reveal (full gsap block verified in the bundle):
//  Each WORD sits in an overflow-hidden mask and rolls UP from translate-y-full with a
//  rotate-6 tilt (stagger .02, natureSway, once) — the "bottom-up, on a tilt" motion.
//  Each CHAR = a dim white/20 ghost + the real char whose opacity 0->1 is scroll-scrubbed
//  (stagger .1, start top 82% -> end top 20%). Accent words settle red (produx's is lime).
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";
const ACCENT_WORDS = new Set(["accountability,", "Casper."]);
const NATURE_SWAY = [0.08, 0.494, 0.14, 1] as const; // produx CustomEase "natureSway"

function Char({ ch, progress, start, accent }: { ch: string; progress: MotionValue<number>; start: number; accent: boolean }) {
	const opacity = useTransform(progress, [start, start + 0.06], [0, 1]);
	return (
		<span className="relative inline-block">
			<span className="absolute top-0 left-0 text-white/20 select-none pointer-events-none" aria-hidden>{ch}</span>
			<motion.span style={{ opacity }} className={accent ? "relative text-[var(--red-700)]" : "relative text-white"}>{ch}</motion.span>
		</span>
	);
}

function Word({ chars, wi }: { chars: React.ReactNode; wi: number }) {
	return (
		<span className="overflow-hidden inline-flex pb-[0.12em]">
			<motion.span
				className="inline-block whitespace-nowrap origin-bottom-left"
				initial={{ y: "100%", rotate: 6, opacity: 0 }}
				whileInView={{ y: "0%", rotate: 0, opacity: 1 }}
				viewport={{ once: true, margin: "0px 0px -12% 0px" }}
				transition={{ duration: 1.1, delay: wi * 0.02, ease: NATURE_SWAY }}
			>
				{chars}
			</motion.span>
		</span>
	);
}

export function ManifestoReveal() {
	const ref = useRef<HTMLDivElement>(null);
	// produx scrub window: start "top 82%" -> end "top 20%"
	const { scrollYProgress } = useScroll({ target: ref, offset: ["start 0.82", "start 0.2"] });
	const words = TEXT.split(" ");
	const total = TEXT.replace(/ /g, "").length;
	let idx = 0;

	return (
		<div ref={ref} className="w-full max-w-[74vw] max-lg:max-w-none">
			<p className="flex flex-wrap gap-x-[0.85vw] gap-y-[0.1em] tracking-tight leading-[1.22]" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, fontSize: "clamp(28px, 3.65vw, 78px)" }}>
				{words.map((word, wi) => {
					const accent = ACCENT_WORDS.has(word);
					const chars = word.split("").map((ch, ci) => {
						const i = idx++;
						const start = (i / total) * 0.86;
						return <Char key={ci} ch={ch} progress={scrollYProgress} start={start} accent={accent} />;
					});
					return <Word key={wi} chars={chars} wi={wi} />;
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
