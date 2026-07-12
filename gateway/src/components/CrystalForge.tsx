"use client";
import { useEffect, useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

// produx's rock section, rebuilt for our crystal. Their rock is a WebGL scene; we use the same
// underlying technique as their canvas path — a scroll-scrubbed frame sequence drawn to a canvas
// with mix-blend-screen (black frame bg -> transparent). Frames = our Veo crystal charging red.
// Values (label design, positions, reveal easings) lifted 1:1 from the produx bundle.
const N = 159;
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
				<div
					className="cf-inner size-[2.85vw] bg-white/10 backdrop-blur-[5px] max-lg:size-[4vw]"
					style={{ scale: 0, opacity: 0, filter: "blur(8px)" }}
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

	useEffect(() => {
		gsap.registerPlugin(ScrollTrigger);
		const canvas = canvasRef.current;
		if (!canvas) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		// preload the sequence (~8MB) LAZILY — only once the section is near the viewport, so it
		// never blocks the initial page load. Draw the nearest loaded frame so scrub never blanks.
		const imgs: HTMLImageElement[] = [];
		let preloaded = false;
		function preload() {
			if (preloaded) return;
			preloaded = true;
			for (let i = 0; i < N; i++) {
				const im = new Image();
				im.src = frameSrc(i);
				imgs[i] = im;
			}
			imgs[0].onload = () => drawFrame(0);
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
		// Sub-frame interpolation: crossfade the current frame toward the next by the fractional part
		// of the scroll position. 159 discrete frames over 360vh step ~20px each, which reads as judder
		// during Lenis's inertia tail; blending adjacent frames makes the motion continuous.
		function drawFrame(idx: number) {
			const f = Math.max(0, Math.min(N - 1, idx));
			const i0 = Math.floor(f);
			const frac = f - i0;
			const a = nearestLoaded(i0);
			if (a < 0) return;
			const imA = imgs[a];
			if (canvas!.width !== imA.naturalWidth) { canvas!.width = imA.naturalWidth; canvas!.height = imA.naturalHeight; }
			ctx!.globalAlpha = 1;
			ctx!.clearRect(0, 0, canvas!.width, canvas!.height);
			ctx!.drawImage(imA, 0, 0);
			if (frac > 0.02 && i0 + 1 < N) {
				const b = nearestLoaded(i0 + 1);
				if (b >= 0 && b !== a) {
					ctx!.globalAlpha = frac;
					ctx!.drawImage(imgs[b], 0, 0);
					ctx!.globalAlpha = 1;
				}
			}
		}

		// Draw the frame for the current scroll position DIRECTLY. Lenis already smooths the scroll;
		// stacking a second frame-lag (lerp) on top made the crystal creep/step a few frames after
		// each gesture as the two lags settled out of sync — that was the "grind". rAF-throttled so we
		// draw at most once per frame with the latest target.
		let pendingFrame = 0, raf = 0;
		function scheduleDraw() { if (!raf) raf = requestAnimationFrame(() => { raf = 0; drawFrame(pendingFrame); }); }

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
				end: "+=360%",
				pin: true,
				anticipatePin: 1,
				invalidateOnRefresh: true,
				onUpdate: (self) => {
					const p = self.progress;
					pendingFrame = p * (N - 1);
					scheduleDraw();
					gsap.set(sparksRef.current, { opacity: Math.max(0, (p - 0.15)) * 1.1 });
					// headline present during the charge, rolls out before the verdict points land
					const wantHead = p >= 0.07 && p < 0.62;
					if (wantHead !== headlineShown) { headlineShown = wantHead; wantHead ? headIn() : headOut(); }
					// the three points pop in on the fully-charged hero, produx-style
					const wantLabels = p >= 0.68;
					if (wantLabels !== labelsShown) { labelsShown = wantLabels; wantLabels ? labelsIn() : labelsOut(); }
				},
			});
		}, sectionRef);

		return () => { io.disconnect(); ctxq.revert(); if (raf) cancelAnimationFrame(raf); };
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
								background: "radial-gradient(circle, rgba(241,50,66,0.9), rgba(241,50,66,0) 70%)",
								animation: `cfDrift ${5 + (i % 5)}s ease-in-out ${(i % 7) * 0.4}s infinite alternate`,
							}}
						/>
					))}
				</div>

				{/* the crystal — scrubbed frame sequence with real alpha (bg keyed out), so it floats
				    with no box. Big portrait hero, sized by height; drop-shadow restores the red halo. */}
				<canvas
					ref={canvasRef}
					className="relative z-[10] h-[82vh] w-auto max-lg:h-[66vh] max-sm:h-[52vh] select-none"
					style={{ filter: "drop-shadow(0 0 55px rgba(241,50,66,0.34)) drop-shadow(0 0 120px rgba(241,50,66,0.18))" }}
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
											background: "linear-gradient(92deg, #ffffff 0%, #f13242 120%)",
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
