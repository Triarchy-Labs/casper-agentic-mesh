"use client";
import React, { useEffect, useRef, useState } from "react";
import { motion, useScroll, useSpring, useMotionValue, useMotionValueEvent, AnimatePresence } from "framer-motion";
import Link from "next/link";
import gsap from "gsap";
import { CustomEase } from "gsap/CustomEase";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(CustomEase, ScrollTrigger);

// produx's signature scroll ease + scrub:1.5 smoothing, extracted from their bundle.
const PX_EASE = [0.2, 0.6, 0.35, 1] as const;

// Animated bracketed nav item — brackets spread out on hover.
function BracketLink({ label, href = "#" }: { label: string; href?: string }) {
	const ref = useRef<HTMLAnchorElement>(null);
	const [hovered, setHovered] = useState(false);
	// Magnetic pull toward the cursor with a springy overshoot (their gsap back.out feel).
	const mx = useSpring(0, { stiffness: 170, damping: 11, mass: 0.4 });
	const my = useSpring(0, { stiffness: 170, damping: 11, mass: 0.4 });

	const onMove = (e: React.MouseEvent) => {
		const el = ref.current;
		if (!el) return;
		const r = el.getBoundingClientRect();
		mx.set((e.clientX - (r.left + r.width / 2)) * 0.35);
		my.set((e.clientY - (r.top + r.height / 2)) * 0.55);
	};
	const reset = () => {
		mx.set(0);
		my.set(0);
		setHovered(false);
	};

	return (
		<motion.a
			ref={ref}
			href={href}
			onMouseEnter={() => setHovered(true)}
			onMouseMove={onMove}
			onMouseLeave={reset}
			style={{
				x: mx,
				y: my,
				display: "inline-flex",
				alignItems: "center",
				gap: 6,
				fontFamily: "ui-monospace, 'Geist Mono', monospace",
				fontSize: "clamp(13px,0.97vw,24px)",
				fontWeight: 200,
				letterSpacing: "0.18em",
				color: hovered ? "#fff" : "rgba(255,255,255,0.66)",
				textDecoration: "none",
				whiteSpace: "nowrap",
			}}
		>
			<motion.span
				animate={{ x: hovered ? -5 : 0, color: hovered ? "#f13242" : "rgba(255,255,255,0.4)" }}
				transition={{ duration: 0.4, ease: PX_EASE }}
			>
				[
			</motion.span>
			{/* roller / drum: label rolls up, a duplicate rolls in from below */}
			<span
				style={{
					position: "relative",
					display: "inline-block",
					overflow: "hidden",
					height: "1.25em",
					lineHeight: "1.25em",
				}}
			>
				<motion.span
					style={{ display: "block" }}
					animate={{ y: hovered ? "-100%" : "0%" }}
					transition={{ duration: 0.42, ease: PX_EASE }}
				>
					{label}
					<span style={{ position: "absolute", left: 0, top: "100%" }}>{label}</span>
				</motion.span>
			</span>
			<motion.span
				animate={{ x: hovered ? 5 : 0, color: hovered ? "#f13242" : "rgba(255,255,255,0.4)" }}
				transition={{ duration: 0.4, ease: PX_EASE }}
			>
				]
			</motion.span>
		</motion.a>
	);
}

export function ScrollHero() {
	const heroRef = useRef<HTMLElement>(null);
	const wordmarkRef = useRef<HTMLDivElement>(null);
	const subtitleRef = useRef<HTMLDivElement>(null);
	const headerRef = useRef<HTMLElement>(null);
	const logoTargetRef = useRef<HTMLSpanElement>(null); // nav "TRIARCHY" — the exact landing slot
	const { scrollY } = useScroll();
	const [scrolled, setScrolled] = useState(false);
	const [menuOpen, setMenuOpen] = useState(false);

	// Cursor-follow with lag + spring bump (their gsap.quickTo/lerp equivalent).
	const rawX = useMotionValue(-200);
	const rawY = useMotionValue(-200);
	const cursorX = useSpring(rawX, { stiffness: 140, damping: 15, mass: 0.5 });
	const cursorY = useSpring(rawY, { stiffness: 140, damping: 15, mass: 0.5 });

	useMotionValueEvent(scrollY, "change", (v) => setScrolled(v > 14));

	useEffect(() => {
		const onMove = (e: MouseEvent) => {
			rawX.set(e.clientX + 20);
			rawY.set(e.clientY + 12);
		};
		window.addEventListener("mousemove", onMove);
		return () => window.removeEventListener("mousemove", onMove);
	}, [rawX, rawY]);

	// On-load entrance: letters rise into place, subtitle fades up under them.
	useEffect(() => {
		CustomEase.create("natureSway", "M0,0 C0.08,0.494 0.14,1 1,1");
		gsap.fromTo(".logo-letter",
			{ y: "110%", opacity: 0 },
			{ y: "0%", opacity: 1, duration: 1.4, stagger: 0.07, delay: 0.6, ease: "natureSway" }
		);
		gsap.fromTo(".logo-subtitle",
			{ y: "20px", opacity: 0 },
			{ y: "0px", opacity: 1, duration: 1.1, delay: 0.9, ease: "power4.out" }
		);
	}, []);

	// produx logo-shrink: the hero pins, and over a long scrub the giant wordmark
	// eases toward the top-left. Our two deviations from produx:
	//   (a) the subtitle dissolves FIRST (first couple ticks),
	//   (b) at the end the wordmark DISSOLVES and our nav panel reveals in its place
	//       (produx keeps the shrunk mark as the logo; we hand off to the nav).
	useEffect(() => {
		if (!heroRef.current) return;
		CustomEase.create("pxScroll", "M0,0 C0.2,0.6 0.35,1 1,1");

		// Exact landing slot: translate + scale so the giant wordmark lands ON the nav
		// "TRIARCHY" (same spot, same width) — so the crossfade reads as one continuous word.
		// Measured live (offsetWidth ignores transforms) and re-read on every ScrollTrigger refresh.
		const measure = () => {
			const nav = logoTargetRef.current;
			const wm = wordmarkRef.current;
			if (!nav || !wm) {
				return { x: -window.innerWidth * 0.32, y: -window.innerHeight * 0.34, s: 0.11 };
			}
			const nr = nav.getBoundingClientRect();
			return {
				x: nr.left + nr.width / 2 - window.innerWidth / 2,
				y: nr.top + nr.height / 2 - window.innerHeight / 2,
				s: nr.width / wm.offsetWidth,
			};
		};

		const ctx = gsap.context(() => {
			const tl = gsap.timeline({
				defaults: { ease: "pxScroll" },
				scrollTrigger: {
					trigger: heroRef.current,
					start: "top top",
					end: "+=185%",
					pin: true,
					scrub: 1.5,
					invalidateOnRefresh: true,
				},
			});
			// 1. subtitle dissolves first (first ~2 ticks), independent of the word
			tl.to(subtitleRef.current, { opacity: 0, y: -26, filter: "blur(6px)", duration: 0.12, ease: "power2.in" }, 0);
			// 2. shrink to the nav wordmark's exact size (smooth throughout)
			tl.to(wordmarkRef.current, { scale: () => measure().s, duration: 0.85, ease: "power1.inOut" }, 0);
			// 3. STRAIGHT, gently-sloped glide toward the nav slot — X and Y share ease + timing,
			//    so the path is a single line (no arc). On wide screens it lands ~30° = shallow.
			//    Gentle in/out so the reposition barely registers; the shrink carries the eye.
			tl.to(wordmarkRef.current, { x: () => measure().x, duration: 0.85, ease: "power1.inOut" }, 0);
			tl.to(wordmarkRef.current, { y: () => measure().y, duration: 0.85, ease: "power1.inOut" }, 0);
			// 4. soft dissolve as the nav emerges — the word just recedes + fades, no felt jump
			tl.to(wordmarkRef.current, { opacity: 0, filter: "blur(8px)", duration: 0.15, ease: "power2.in" }, 0.82);
			// 5. nav reveals FROM FOG, in place (blur->0 + fade, no slide) — produx reveal 1:1
			tl.fromTo(
				headerRef.current,
				{ autoAlpha: 0, filter: "blur(10px)" },
				{ autoAlpha: 1, filter: "blur(0px)", duration: 0.18, ease: "power2.out" },
				0.82
			);
		}, heroRef);

		// Fonts change the measured widths — re-measure once they load so the landing is exact.
		if (typeof document !== "undefined" && document.fonts) {
			document.fonts.ready.then(() => ScrollTrigger.refresh());
		}
		return () => ctx.revert();
	}, []);

	return (
		<>
			<AnimatePresence>
				{menuOpen && (
					<motion.div
						initial={{ height: 0 }}
						animate={{ height: "100vh" }}
						exit={{ height: 0 }}
						transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
						className="px-[5.5vw] pt-[11.7vh] pb-[4vh] max-lg:px-[4.10vw] max-lg:pt-[11.39vh] max-sm:px-[5.97vw]"
						style={{
							position: "fixed",
							top: 0,
							left: 0,
							right: 0,
							zIndex: 90,
							background: "#020103",
							display: "flex",
							flexDirection: "column",
							justifyContent: "space-between",
							color: "#fff",
							overflow: "hidden",
						}}
					>
						<div className="flex flex-col gap-6 text-[7.9vw] font-bold uppercase tracking-tight mt-12">
							<Link href="/" onClick={() => setMenuOpen(false)}>
								<span className="hover:text-[var(--red-700)] transition-colors cursor-pointer">HOME</span>
							</Link>
							<Link href="/bounties" onClick={() => setMenuOpen(false)}>
								<span className="hover:text-[var(--red-700)] transition-colors cursor-pointer">BOUNTIES</span>
							</Link>
							<Link href="/dashboard" onClick={() => setMenuOpen(false)}>
								<span className="hover:text-[var(--red-700)] transition-colors cursor-pointer">DASHBOARD</span>
							</Link>
						</div>
						<div className="text-sm opacity-50 font-mono uppercase tracking-widest">
							ECONOMIC OS FOR THE AGENT ECONOMY · CASPER
						</div>
					</motion.div>
				)}
			</AnimatePresence>

			{/* cursor-follow [ SCROLL DOWN ] label — vanishes on first scroll */}
			<motion.div
				aria-hidden
				animate={{ opacity: scrolled ? 0 : 0.9 }}
				transition={{ duration: 0.3 }}
				style={{
					position: "fixed",
					left: 0,
					top: 0,
					x: cursorX,
					y: cursorY,
					zIndex: 9999,
					pointerEvents: "none",
					fontFamily: "ui-monospace, 'Geist Mono', monospace",
					fontSize: "clamp(11px,0.95vw,30px)",
					letterSpacing: "0.18em",
					textTransform: "uppercase",
					color: "#fff",
					mixBlendMode: "difference", // reactive color inversion vs background (produx)
					whiteSpace: "nowrap",
					padding: "0.23vw 0.55vw",
				}}
			>
				[ scroll down ]
			</motion.div>

			{/* Nav panel — hidden at first; GSAP reveals it as the wordmark dissolves. */}
			<header
				ref={headerRef}
				className="fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-[5.5vw] pt-[8vh] max-lg:px-[4.10vw] max-lg:pt-[9.59vh] max-sm:px-[5.97vw] max-sm:pt-[11.3vh]"
				style={{
					opacity: 0,
					visibility: "hidden",
					fontFamily: "ui-monospace, 'Geist Mono', monospace",
					pointerEvents: "auto",
				}}
			>
				<span style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, letterSpacing: "0.16em", fontSize: "clamp(14px,1.1vw,24px)" }}>
					<span ref={logoTargetRef} style={{ display: "inline-block" }}>TRIARCHY</span>{" "}
					<motion.span
						style={{ color: "#f13242", display: "inline-block" }}
						animate={{
							scale: [1, 1.09, 1],
							textShadow: [
								"0 0 0px rgba(241,50,66,0)",
								"0 0 11px rgba(241,50,66,0.8)",
								"0 0 0px rgba(241,50,66,0)",
							],
						}}
						transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut" }}
					>
						/
					</motion.span>
					<motion.span
						style={{ color: "#f13242", display: "inline-block" }}
						animate={{
							scale: [1, 1.09, 1],
							textShadow: [
								"0 0 0px rgba(241,50,66,0)",
								"0 0 11px rgba(241,50,66,0.8)",
								"0 0 0px rgba(241,50,66,0)",
							],
						}}
						transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut", delay: 2.4 }}
					>
						/
					</motion.span>{" "}
					MESH
				</span>
				<nav className="flex items-center gap-[4.1vw]">
					<div className="flex items-center gap-[4.1vw] max-lg:hidden">
						<BracketLink label="BOUNTIES" href="/bounties" />
						<BracketLink label="DASHBOARD" href="/dashboard" />
					</div>
					<div className="flex items-center gap-[4.1vw]">
						<BracketLink label="WALLET" />
						<div onClick={() => setMenuOpen(!menuOpen)} className="z-[100]">
							<BracketLink label={menuOpen ? "CLOSE" : "MENU"} />
						</div>
					</div>
				</nav>
			</header>

			{/* giant scroll-shrink wordmark — pinned first screen (produx pin + scrub) */}
			<section ref={heroRef} style={{ height: "100vh", position: "relative", overflow: "hidden" }}>
				<div
					style={{
						position: "absolute",
						inset: 0,
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					<div
						ref={wordmarkRef}
						style={{
							position: "relative",
							transformOrigin: "center center",
							textAlign: "center",
							pointerEvents: "none",
							willChange: "transform, opacity, filter",
						}}
					>
						<div
							style={{
								fontFamily: "var(--font-tech), 'Sora', sans-serif",
								fontWeight: 800,
								fontSize: "clamp(72px, 11.9vw, 900px)", // produx hero = 11.9vw pure (no low cap on 4K)
								lineHeight: 0.88,
								letterSpacing: "-0.035em",
								color: "#fff",
								textShadow: "0 8px 60px rgba(0,0,0,0.6)",
							}}
						>
							{"TRIARCHY".split("").map((letter, i) => (
								<span key={i} className="relative inline-block overflow-hidden py-4 -my-4">
									<span className="logo-letter inline-block translate-y-full opacity-0">
										{letter}
									</span>
								</span>
							))}
						</div>
						<div
							ref={subtitleRef}
							className="neon-sweep logo-subtitle opacity-0"
							style={{
								position: "absolute",
								top: "100%",
								left: 0,
								right: 0,
								marginTop: "1.4rem",
								textAlign: "center",
								whiteSpace: "nowrap",
								fontFamily: "ui-monospace, 'Geist Mono', monospace",
								letterSpacing: "0.36em",
								fontSize: "clamp(12px,1.5vw,64px)",
							}}
						>
							ECONOMIC OS FOR THE AGENT ECONOMY{" "}
							<span style={{ WebkitTextFillColor: "#f13242", color: "#f13242" }}>·</span> CASPER
						</div>
					</div>
				</div>
			</section>
		</>
	);
}
