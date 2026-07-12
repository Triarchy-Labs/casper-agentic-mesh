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
const HEADLINE = [
	["Forged", "where", "human"],
	["and", "machine", "meet"],
	["— trust,", "written", "in", "code,"],
	["settled", "on", "Casper."],
];

// the three service-points (produx design + positions), our product truths
const POINTS = [
	{ pos: "top-[15%] right-[24%]", size: "size-[3.75vw] max-lg:size-[5.27vw]", text: "Value locked in code", side: "left" as const },
	{ pos: "top-[40%] left-[16.5%]", size: "size-[4.51vw] max-lg:size-[6.34vw]", text: "Verdicts final, on-chain", side: "right" as const },
	{ pos: "top-[68%] right-[8%]", size: "size-[3.75vw] max-lg:size-[5.27vw]", text: "Settled on Casper testnet", side: "left" as const },
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
		function drawFrame(idx: number) {
			let i = Math.max(0, Math.min(N - 1, Math.round(idx)));
			if (!imgs[i]?.complete) {
				for (let d = 1; d < N; d++) {
					if (imgs[Math.max(0, i - d)]?.complete) { i = Math.max(0, i - d); break; }
					if (imgs[Math.min(N - 1, i + d)]?.complete) { i = Math.min(N - 1, i + d); break; }
				}
			}
			const im = imgs[i];
			if (!im || !im.complete || !im.naturalWidth) return;
			if (canvas!.width !== im.naturalWidth) { canvas!.width = im.naturalWidth; canvas!.height = im.naturalHeight; }
			ctx!.clearRect(0, 0, canvas!.width, canvas!.height);
			ctx!.drawImage(im, 0, 0);
		}

		// scrub-lag so frames glide toward the scroll target (same buttery feel as the rest of the site)
		let target = 0, cur = 0, raf = 0;
		function tick() {
			cur += (target - cur) * 0.14;
			if (Math.abs(target - cur) < 0.35) { cur = target; drawFrame(cur); raf = 0; return; }
			drawFrame(cur);
			raf = requestAnimationFrame(tick);
		}
		function nudge() { if (!raf) raf = requestAnimationFrame(tick); }

		const ctxq = gsap.context(() => {
			let headlineShown = false, labelsShown = false;
			const headIn = () => gsap.to(".cf-word", { yPercent: 0, opacity: 1, filter: "blur(0px)", duration: 0.8, ease: "power4.out", stagger: 0.03 });
			const headOut = () => gsap.to(".cf-word", { yPercent: 100, opacity: 0, filter: "blur(6px)", duration: 0.6, ease: "expo.in", stagger: 0.02 });
			const labelsIn = () => {
				gsap.fromTo(".cf-square", { scale: 0, opacity: 0 }, { scale: 1, opacity: 1, duration: 1.2, ease: "expo.out", stagger: 0.1 });
				gsap.fromTo(".cf-inner", { scale: 0, opacity: 0, filter: "blur(8px)" }, { scale: 1, opacity: 1, filter: "blur(0px)", duration: 1.2, ease: "expo.out", stagger: 0.1 });
				gsap.to(".cf-label-word", { yPercent: 0, opacity: 1, duration: 0.8, ease: "power4.out", stagger: 0.02, delay: 0.1 });
			};
			const labelsOut = () => {
				gsap.to(".cf-square, .cf-inner", { scale: 0, opacity: 0, duration: 1, ease: "expo.in" });
				gsap.to(".cf-label-word", { yPercent: 100, opacity: 0, duration: 0.5, ease: "expo.in" });
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
					target = p * (N - 1);
					nudge();
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
