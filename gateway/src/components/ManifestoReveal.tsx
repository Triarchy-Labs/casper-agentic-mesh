"use client";
import { Fragment, useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// produx's manifesto reveal, ported: text is split per character (<span class="char">),
// and a wave of "reveal" runs through it on scroll-scrub — each char goes
// dim -> ACCENT (the glitching leading edge) -> settled, with a small positional glitch.
// produx's edge is lime-green; ours is brand red.
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";
const ACCENT_WORDS = new Set(["accountability,", "Casper."]); // settle red instead of white

const RED = "#f13242";
const DIM = "rgba(190,190,196,0.22)";

function Char({ ch, progress, start, end, accent }: { ch: string; progress: MotionValue<number>; start: number; end: number; accent: boolean }) {
	const p = useTransform(progress, [start, end], [0, 1]); // this char's local 0..1
	// dim grey -> red flash (the decoding edge) -> white (or red for accent words)
	const color = useTransform(p, [0, 0.45, 1], [DIM, RED, accent ? RED : "#ffffff"]);
	const opacity = useTransform(p, [0, 0.12], [0.28, 1]);
	// unstable "datamosh" jitter that resolves as the char settles
	const x = useTransform(p, [0, 0.18, 0.45, 0.72, 1], [0, -2.4, 2.2, -1, 0]);
	const skewX = useTransform(p, [0, 0.22, 0.55, 1], [0, -9, 5, 0]);
	return (
		<motion.span style={{ color, opacity, x, skewX, display: "inline-block", willChange: "transform, color, opacity" }}>
			{ch}
		</motion.span>
	);
}

function DecodeReveal({ text, progress }: { text: string; progress: MotionValue<number> }) {
	const words = text.split(" ");
	const total = text.replace(/ /g, "").length;
	const span = 0.72; // whole reveal completes within the first 72% of the scroll range
	let idx = 0;
	return (
		<>
			{words.map((word, wi) => {
				const accent = ACCENT_WORDS.has(word);
				return (
					<Fragment key={wi}>
						<span style={{ display: "inline-block", whiteSpace: "nowrap" }}>
							{word.split("").map((ch, ci) => {
								const i = idx++;
								const start = (i / total) * span;
								return <Char key={ci} ch={ch} progress={progress} start={start} end={start + 0.16} accent={accent} />;
							})}
						</span>
						{wi < words.length - 1 ? " " : ""}
					</Fragment>
				);
			})}
		</>
	);
}

export function ManifestoReveal() {
	const ref = useRef<HTMLDivElement>(null);
	const { scrollYProgress } = useScroll({ target: ref, offset: ["start 0.85", "start 0.15"] });

	return (
		<div ref={ref} className="w-full max-w-[74vw] max-lg:max-w-none">
			<p className="tracking-tight leading-[1.18]" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, fontSize: "clamp(28px, 3.65vw, 78px)" }}>
				<DecodeReveal text={TEXT} progress={scrollYProgress} />
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
