"use client";
import { useEffect, useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { useLenis } from "lenis/react";
import type Lenis from "lenis";

// produx's rock section, rebuilt for our crystal. Their rock is a WebGL scene; we use the same
// underlying technique as their canvas path — a scroll-scrubbed frame sequence drawn to a canvas
// with mix-blend-screen (black frame bg -> transparent). Frames = our Veo crystal charging red.
// Values (label design, positions, reveal easings) lifted 1:1 from the produx bundle.
const N = 472;
const frameSrc = (i: number) => `/crystal/f_${String(i + 1).padStart(3, "0")}.webp`;

// gradient headline, produx cadence — grouped words, masked roll-up
// the phrase we agreed (user-proposed, refined): "Forged in an accelerating current of time —
// at the seam of human and machine. Web3 has never been this solid, this clean." Headline carries
// the core; the three points carry the continuation + the product anchor.
const HEADLINE = [
	["Forged", "in", "an"],
	["accelerating", "current"],
	["of", "time."],
];

// the three service-points (produx design + positions), our agreed phrase
const POINTS = [
	{ pos: "top-[15%] right-[24%]", size: "size-[3.75vw] max-lg:size-[5.27vw]", text: "At the seam of human & machine", side: "left" as const },
	{ pos: "top-[40%] left-[16.5%]", size: "size-[4.51vw] max-lg:size-[6.34vw]", text: "Web3, never this solid — this clean", side: "right" as const },
	{ pos: "top-[68%] right-[8%]", size: "size-[3.75vw] max-lg:size-[5.27vw]", text: "Settled on Casper", side: "left" as const },
];

function Caption({ text }: { text: string }) {
	return (
		<div
			className="text-white/85 flex h-fit max-w-[14vw] flex-wrap items-start gap-x-[0.3em] gap-y-[0.2em] text-[1.66vw] leading-[1.2] font-light max-lg:max-w-[20vw] max-lg:text-[2.34vw]"
			style={{ fontFamily: "var(--font-DM-mono), var(--font-mono), monospace" }}
		>
			{text.split(" ").map((w, i) => (
				<span key={i} className="inline-flex overflow-hidden" style={{ lineHeight: 1.2 }}>
					<span className="cf-label-word block translate-y-full opacity-0">{w}</span>
				</span>
			))}
		</div>
	);
}

function ServicePoint({ p }: { p: (typeof POINTS)[number] }) {
	const square = (
		<div className={`relative ${p.size}`}>
			<div className="cf-square absolute inset-0 border border-[#F2F2F2]" style={{ scale: 0, opacity: 0 }} />
			<div className="absolute inset-0 flex items-center justify-center">
				{/* NO backdrop-blur: it recomposites the (animating) crystal behind it every frame and
				    was the main cause of the scroll judder. Plain translucent fill reads the same. */}
				<div
					className="cf-inner size-[2.85vw] bg-white/[0.14] max-lg:size-[4vw]"
					style={{ scale: 0, opacity: 0 }}
				/>
			</div>
		</div>
	);
	return (
		<div className={`absolute ${p.pos} z-20 flex w-fit items-center gap-[0.97vw] max-sm:hidden`}>
			{p.side === "left" ? (
				<>
					{square}
					<Caption text={p.text} />
				</>
			) : (
				<>
					<Caption text={p.text} />
					{square}
				</>
			)}
		</div>
	);
}

export function CrystalForge() {
	const sectionRef = useRef<HTMLDivElement>(null);
	const pinRef = useRef<HTMLDivElement>(null);
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const sparksRef = useRef<HTMLDivElement>(null);
	const glowRef = useRef<HTMLDivElement>(null);
	// Lenis instance (updated every render) — the settle logic reads targetScroll from it lazily.
	const lenisRef = useRef<Lenis | null>(null);
	const lenis = useLenis();
	if (lenis) lenisRef.current = lenis;

	useEffect(() => {
		gsap.registerPlugin(ScrollTrigger);
		const canvas = canvasRef.current;
		if (!canvas) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		// TRICKLE preload (~27MB, 472 frames). The old one-shot loop fired all 472 Image()s in a
		// single synchronous burst right around the cards area (rootMargin) — network + decode
		// stampede on the main thread = the first-visit micro-stall the user felt. produx avoids
		// this class of jank because their heavy media is a streaming <video> (progressive,
		// off-thread decode). Ours: max 12 concurrent, COARSE-FIRST order (every 8th frame, then
		// 4th, 2nd, all) so the scrub works almost immediately at low density and refines while
		// you approach; nearestLoaded() already bridges the gaps.
		const imgs: HTMLImageElement[] = [];
		let preloaded = false;
		function preload() {
			if (preloaded) return;
			preloaded = true;
			const queue: number[] = [];
			const seen = new Set<number>();
			for (const step of [8, 4, 2, 1]) {
				for (let i = 0; i < N; i += step) {
					if (!seen.has(i)) { seen.add(i); queue.push(i); }
				}
			}
			let qi = 0, inflight = 0;
			const MAXC = 12;
			const pump = () => {
				while (inflight < MAXC && qi < queue.length) {
					const i = queue[qi++];
					const im = new Image();
					im.decoding = "async";
					imgs[i] = im;
					inflight++;
					const done = () => { inflight--; if (i === 0) drawFrame(0); pump(); };
					im.onload = done;
					im.onerror = done;
					im.src = frameSrc(i);
				}
			};
			pump();
		}
		const io = new IntersectionObserver(
			(entries) => { if (entries[0].isIntersecting) { preload(); io.disconnect(); } },
			{ rootMargin: "2800px 0px" }
		);
		if (sectionRef.current) io.observe(sectionRef.current);
		function nearestLoaded(i: number): number {
			if (imgs[i]?.complete && imgs[i].naturalWidth) return i;
			for (let d = 1; d < N; d++) {
				const lo = Math.max(0, i - d), hi = Math.min(N - 1, i + d);
				if (imgs[lo]?.complete && imgs[lo].naturalWidth) return lo;
				if (imgs[hi]?.complete && imgs[hi].naturalWidth) return hi;
			}
			return -1;
		}
		// Draw one sharp frame. Smoothness = frame DENSITY (472 optical-flow interpolated frames), not
		// blur/blend — each step is a real intermediate rotation, sharp.
		function drawFrame(idx: number) {
			const i = nearestLoaded(Math.round(Math.max(0, Math.min(N - 1, idx))));
			if (i < 0) return;
			const im = imgs[i];
			if (canvas!.width !== im.naturalWidth) { canvas!.width = im.naturalWidth; canvas!.height = im.naturalHeight; }
			ctx!.clearRect(0, 0, canvas!.width, canvas!.height);
			ctx!.drawImage(im, 0, 0);
		}

		// NO separate requestAnimationFrame for drawing. The frame is drawn synchronously inside the
		// ScrollTrigger onUpdate, which fires from ScrollTrigger.update() — already driven by gsap.ticker
		// (see SmoothScroller). So the canvas draw lands in the SAME tick as the Lenis scroll update,
		// killing the 1-frame desync that caused the micro-jitter on the settle. Density (472 interpolated
		// frames) + Lenis's own smoothing give the smoothness; no extra lag needed.

		// Settle logic: while scrolling fast the frame tracks scroll 1:1. Once velocity drops below
		// SETTLE_V (gesture released, Lenis coasting), we DON'T ride the slow coast tail frame-by-frame
		// (that's the visible end-ticks) — instead we read Lenis's final destination (targetScroll),
		// and glide the remaining frames there in ONE short eased tween (~0.35s ≈ 30+fps = fluid).
		const SETTLE_V = 250; // px/s
		const frameProxy = { f: 0 };
		let settleTween: gsap.core.Tween | null = null;
		let settleTarget = -1;
		const ctxq = gsap.context(() => {
			let headlineShown = false, labelsShown = false;
			// produx uses gsap `y:"0%"` (NOT yPercent): `y` reads the element's current transform matrix,
			// so it correctly animates FROM the Tailwind `translate-y-full` (translateY 100%) baseline.
			// yPercent is a separate gsap prop with a 0 baseline -> it never moved the words off-screen.
			const headIn = () => gsap.to(".cf-word", { y: "0%", opacity: 1, filter: "blur(0px)", duration: 0.8, ease: "power4.out", stagger: 0.03, overwrite: "auto" });
			const headOut = () => gsap.to(".cf-word", { y: "100%", opacity: 0, filter: "blur(6px)", duration: 0.6, ease: "expo.in", stagger: 0.02, overwrite: "auto" });
			const labelsIn = () => {
				gsap.fromTo(".cf-square", { scale: 0, opacity: 0 }, { scale: 1, opacity: 1, duration: 1.2, ease: "expo.out", stagger: 0.1, overwrite: "auto" });
				gsap.fromTo(".cf-inner", { scale: 0, opacity: 0, filter: "blur(8px)" }, { scale: 1, opacity: 1, filter: "blur(0px)", duration: 1.2, ease: "expo.out", stagger: 0.1, overwrite: "auto" });
				gsap.to(".cf-label-word", { y: "0%", opacity: 1, filter: "blur(0px)", duration: 0.8, ease: "power4.out", stagger: 0.03, delay: 0.1, overwrite: "auto" });
			};
			const labelsOut = () => {
				gsap.to(".cf-square, .cf-inner", { scale: 0, opacity: 0, duration: 1, ease: "expo.in", overwrite: "auto" });
				gsap.to(".cf-label-word", { y: "100%", opacity: 0, filter: "blur(6px)", duration: 0.5, ease: "expo.in", overwrite: "auto" });
			};

			ScrollTrigger.create({
				trigger: pinRef.current,
				start: "top top",
				// ~1.3x more scroll for the same rotation (360% -> 470%): slower per-gesture crystal
				// travel = fewer frame boundaries crossed as Lenis's inertia settles = smoother stop.
				end: "+=470%",
				pin: true,
				anticipatePin: 1,
				invalidateOnRefresh: true,
				onUpdate: (self) => {
					const p = self.progress;
					const v = Math.abs(self.getVelocity());
					if (v > SETTLE_V) {
						// actively scrolling: track 1:1, kill any settle glide
						if (settleTween) { settleTween.kill(); settleTween = null; settleTarget = -1; }
						frameProxy.f = p * (N - 1);
						drawFrame(frameProxy.f);
					} else {
						// coasting: glide the frames to Lenis's final destination in one smooth move
						const L = lenisRef.current;
						const span = self.end - self.start;
						const finalP = L && span > 0
							? Math.max(0, Math.min(1, (L.targetScroll - self.start) / span))
							: p;
						const targetF = finalP * (N - 1);
						if (Math.abs(targetF - frameProxy.f) < 0.5) {
							// already there — just settle exactly
							if (!settleTween) { frameProxy.f = targetF; drawFrame(targetF); }
						} else if (settleTarget < 0 || Math.abs(targetF - settleTarget) > 0.75) {
							settleTarget = targetF;
							if (settleTween) settleTween.kill();
							settleTween = gsap.to(frameProxy, {
								f: targetF,
								duration: 0.35,
								ease: "power3.out",
								onUpdate: () => drawFrame(frameProxy.f),
								onComplete: () => { settleTween = null; settleTarget = -1; },
							});
						}
					}
					const charge = Math.max(0, (p - 0.15)) * 1.1;
					gsap.set(sparksRef.current, { opacity: charge });
					gsap.set(glowRef.current, { opacity: Math.min(1, charge) });
					// headline present during the charge, rolls out before the verdict points land
					const wantHead = p >= 0.07 && p < 0.62;
					if (wantHead !== headlineShown) {
						headlineShown = wantHead;
						if (wantHead) headIn(); else headOut();
					}
					// the three points pop in on the fully-charged hero, produx-style
					const wantLabels = p >= 0.68;
					if (wantLabels !== labelsShown) {
						labelsShown = wantLabels;
						if (wantLabels) labelsIn(); else labelsOut();
					}
				},
			});
		}, sectionRef);

		return () => { io.disconnect(); if (settleTween) settleTween.kill(); ctxq.revert(); };
	}, []);

	return (
		<section ref={sectionRef} className="relative w-full">
			<div ref={pinRef} className="relative h-screen w-full overflow-hidden flex items-center justify-center">
				{/* red sparks (produx's #bbc868 particles -> our embers), charge-gated */}
				<div ref={sparksRef} className="pointer-events-none absolute inset-0 z-[15]" style={{ opacity: 0 }}>
					{Array.from({ length: 16 }).map((_, i) => (
						<span
							key={i}
							className="cf-spark absolute rounded-full"
							style={{
								left: `${12 + ((i * 53) % 76)}%`,
								top: `${14 + ((i * 37) % 70)}%`,
								width: `${3 + (i % 3)}px`,
								height: `${3 + (i % 3)}px`,
								background: "radial-gradient(circle, rgba(224,53,41,0.9), rgba(224,53,41,0) 70%)",
								animation: `cfDrift ${5 + (i % 5)}s ease-in-out ${(i % 7) * 0.4}s infinite alternate`,
							}}
						/>
					))}
				</div>

				{/* STATIC red halo behind the crystal — a radial-gradient div (composited once), not a
				    per-frame canvas drop-shadow filter (which recomputed every frame the crystal drew
				    and fed the judder). Its opacity ramps with the charge via a cheap gsap.set. */}
				<div
					ref={glowRef}
					className="pointer-events-none absolute z-[8] h-[70vh] w-[46vw] max-lg:w-[64vw] max-sm:w-[88vw]"
					style={{ opacity: 0, background: "radial-gradient(ellipse at center, rgba(224,53,41,0.42), rgba(224,53,41,0.12) 45%, transparent 72%)" }}
				/>

				{/* the crystal — scrubbed frame sequence with real alpha (bg keyed out), floats with no box */}
				<canvas
					ref={canvasRef}
					className="relative z-[10] h-[82vh] w-auto max-lg:h-[66vh] max-sm:h-[52vh] select-none"
					aria-hidden
				/>

				{/* gradient headline, masked word roll-up */}
				<h2 className="absolute bottom-[12vh] left-[5.5vw] z-20 max-w-[42vw] uppercase leading-[1.06] max-lg:max-w-[62vw] max-sm:bottom-[8vh]"
					style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(26px, 3.5vw, 68px)", fontWeight: 400 }}>
					{HEADLINE.map((group, gi) => (
						<span key={gi} className="mr-[0.3em] inline-flex flex-wrap">
							{group.map((w, wi) => (
								<span key={wi} className="mr-[0.25em] inline-flex overflow-hidden">
									<span
										className="cf-word block translate-y-full opacity-0"
										style={{
											background: "linear-gradient(92deg, #ffffff 0%, #e03529 120%)",
											WebkitBackgroundClip: "text",
											backgroundClip: "text",
											WebkitTextFillColor: "transparent",
										}}
									>
										{w}
									</span>
								</span>
							))}
						</span>
					))}
				</h2>

				{POINTS.map((p, i) => (
					<ServicePoint key={i} p={p} />
				))}
			</div>
		</section>
	);
}
