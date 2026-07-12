"use client";
import { Fragment, useRef } from "react";
import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";

// produx's manifesto reveal, ported. Each character has two layers:
//  1) a base glyph that colour-shifts dim -> settled (white, or red for accent words);
//  2) a RED pixel-dither overlay whose "wipe" edge sweeps across the glyph on scroll —
//     a fine pixel grid intersected with a moving hard edge, so the glyph materialises in
//     pixel chunks (produx does this in green; ours is brand red), then fades to the clean base.
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";
const ACCENT_WORDS = new Set(["accountability,", "Casper."]);

const RED = "#f13242";
const DIM = "rgba(190,190,196,0.22)";
// pixel grid (checker) + a moving hard wipe edge; intersected -> pixelated reveal.
// wipe runs BOTTOM-TO-TOP on a diagonal (-20deg), like produx; bigger cells = clearer square pixels.
const MASK = "linear-gradient(-20deg, #000 var(--wipe, 0%), transparent var(--wipe, 0%)), repeating-conic-gradient(#000 0% 25%, transparent 0% 50%)";
const MASK_SIZE = "100% 100%, 6px 6px";

function Char({ ch, progress, start, end, accent }: { ch: string; progress: MotionValue<number>; start: number; end: number; accent: boolean }) {
	const p = useTransform(progress, [start, end], [0, 1]);
	const color = useTransform(p, [0.2, 0.9], [DIM, accent ? RED : "#ffffff"]);
	const x = useTransform(p, [0, 0.22, 0.5, 0.8, 1], [0, -2.2, 2, -1, 0]);
	const wipe = useTransform(p, [0, 1], ["-15%", "120%"]);
	const pixOpacity = useTransform(p, [0, 0.12, 0.72, 1], [0, 1, 1, 0]);

	return (
		<span style={{ position: "relative", display: "inline-block" }}>
			<motion.span style={{ color, x, display: "inline-block", willChange: "transform, color" }}>{ch}</motion.span>
			<motion.span
				aria-hidden
				style={{
					position: "absolute",
					left: 0,
					top: 0,
					color: RED,
					x,
					opacity: pixOpacity,
					pointerEvents: "none",
					userSelect: "none",
					display: "inline-block",
					maskImage: MASK,
					WebkitMaskImage: MASK,
					maskSize: MASK_SIZE,
					WebkitMaskSize: MASK_SIZE,
					maskComposite: "intersect",
					WebkitMaskComposite: "source-in",
					// @ts-expect-error CSS var driven by a motion value
					"--wipe": wipe,
				}}
			>
				{ch}
			</motion.span>
		</span>
	);
}

function DecodeReveal({ text, progress }: { text: string; progress: MotionValue<number> }) {
	const words = text.split(" ");
	const total = text.replace(/ /g, "").length;
	const span = 0.72;
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
