"use client";
import { Fragment, useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// Ported 1:1 from produx's journal reveal (verified in their bundle):
//   per char: a dim ghost (text-white/20) always visible + the real char (opacity 0 -> 1)
//   reveal: gsap.to('.char', { opacity:1, ease:'none', stagger:.1, scrollTrigger scrub .5 })
//   words enter with translate-y-full rotate-6 opacity-0.
// produx's accent is lime; ours is brand red on the key words.
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";
const ACCENT_WORDS = new Set(["accountability,", "Casper."]);

function Char({ ch, progress, start, accent }: { ch: string; progress: MotionValue<number>; start: number; accent: boolean }) {
	const opacity = useTransform(progress, [start, start + 0.07], [0, 1]);
	return (
		<span className="relative inline-block">
			<span className="absolute top-0 left-0 text-white/20 select-none pointer-events-none" aria-hidden>{ch}</span>
			<motion.span style={{ opacity }} className={accent ? "relative text-[var(--red-700)]" : "relative text-white"}>{ch}</motion.span>
		</span>
	);
}

function Word({ children, i }: { children: React.ReactNode; i: number }) {
	return (
		<motion.span
			className="inline-block whitespace-nowrap mr-[0.26em] align-baseline"
			initial={{ y: "42%", rotate: 5, opacity: 0 }}
			whileInView={{ y: "0%", rotate: 0, opacity: 1 }}
			viewport={{ once: true, amount: 0.4 }}
			transition={{ duration: 0.7, delay: i * 0.03, ease: [0.2, 0.8, 0.2, 1] }}
		>
			{children}
		</motion.span>
	);
}

export function ManifestoReveal() {
	const ref = useRef<HTMLDivElement>(null);
	// produx trigger: start "top 82%" -> end "top 20%", scrub .5
	const { scrollYProgress } = useScroll({ target: ref, offset: ["start 0.82", "start 0.2"] });
	const words = TEXT.split(" ");
	const total = TEXT.replace(/ /g, "").length;
	let idx = 0;

	return (
		<div ref={ref} className="w-full max-w-[74vw] max-lg:max-w-none">
			<p className="tracking-tight leading-[1.3]" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, fontSize: "clamp(28px, 3.65vw, 78px)" }}>
				{words.map((word, wi) => {
					const accent = ACCENT_WORDS.has(word);
					return (
						<Word key={wi} i={wi}>
							{word.split("").map((ch, ci) => {
								const i = idx++;
								const start = (i / total) * 0.86;
								return <Char key={ci} ch={ch} progress={scrollYProgress} start={start} accent={accent} />;
							})}
						</Word>
					);
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
