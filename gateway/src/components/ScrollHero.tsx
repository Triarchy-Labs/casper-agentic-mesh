"use client";
import React, { useEffect, useRef, useState } from "react";
import { motion, useScroll, useTransform, useSpring, useMotionValue, useMotionValueEvent } from "framer-motion";
import gsap from "gsap";
import { CustomEase } from "gsap/CustomEase";

gsap.registerPlugin(CustomEase);

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
				fontSize: "clamp(13px,1.2vw,58px)",
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
	const { scrollY } = useScroll();
	// Emulate ScrollTrigger scrub:1.5 — a soft, slightly longer lag behind the raw
	// scroll so the wordmark eases into the nav gradually, never in jerks.
	const smoothY = useSpring(scrollY, { stiffness: 38, damping: 24, restDelta: 0.5 });
	const [scrolled, setScrolled] = useState(false);

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

	useEffect(() => {
		CustomEase.create("natureSway", "M0,0 C0.08,0.494 0.14,1 1,1");
		gsap.fromTo(".logo-letter",
			{ y: "110%", opacity: 0 },
			{
				y: "0%",
				opacity: 1,
				duration: 1.4,
				stagger: 0.07,
				delay: 0.6,
				ease: "natureSway",
			}
		);
		gsap.fromTo(".logo-subtitle",
			{ y: "20px", opacity: 0 },
			{
				y: "0px",
				opacity: 1,
				duration: 1.1,
				delay: 0.9,
				ease: "power4.out",
			}
		);
	}, []);

	// Giant wordmark shrinks + rises + fades over the first ~520px of scroll.
	const wmScale = useTransform(smoothY, [0, 740], [1, 0.2]);
	const wmY = useTransform(smoothY, [0, 740], [0, -160]);
	const wmOpacity = useTransform(smoothY, [0, 480, 740], [1, 0.2, 0]);
	const blur = useTransform(smoothY, [420, 740], [0, 9]);
	const wmFilter = useTransform(blur, (b) => `blur(${b}px)`);

	// Docked bracket nav fades in as the wordmark shrinks away.
	const navOpacity = useTransform(smoothY, [220, 520], [0, 1]);
	const navY = useTransform(smoothY, [220, 520], [-18, 0]);

	return (
		<>
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

			{/* fixed top bar: docked logo + animated bracket nav (fades in on scroll) */}
			<motion.header
				style={{
					opacity: navOpacity,
					y: navY,
					position: "fixed",
					top: 0,
					left: 0,
					right: 0,
					zIndex: 50,
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between",
					padding: "16px clamp(20px,4vw,56px)",
					fontFamily: "ui-monospace, 'Geist Mono', monospace",
				}}
			>
				<span style={{ fontWeight: 700, letterSpacing: "0.16em", fontSize: "clamp(16px,1.9vw,90px)" }}>
					TRIARCHY{" "}
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
				<nav style={{ display: "flex", gap: "clamp(16px,2.6vw,90px)", alignItems: "center" }}>
					<BracketLink label="BOUNTIES" href="/bounties" />
					<BracketLink label="DASHBOARD" href="/dashboard" />
					<BracketLink label="CONNECT WALLET" />
				</nav>
			</motion.header>

			{/* giant scroll-shrink wordmark — first screen */}
			<section style={{ height: "220vh", position: "relative" }}>
				{/* sticky pin: the hero holds in place while the wordmark shrinks (produx pin) */}
				<div
					style={{
						position: "sticky",
						top: 0,
						height: "100vh",
						display: "flex",
						flexDirection: "column",
						alignItems: "center",
						justifyContent: "center",
						overflow: "hidden",
					}}
				>
				<motion.div
					style={{
						scale: wmScale,
						y: wmY,
						opacity: wmOpacity,
						filter: wmFilter,
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
						className="neon-sweep logo-subtitle opacity-0"
						style={{
							marginTop: "1.4rem",
							fontFamily: "ui-monospace, 'Geist Mono', monospace",
							letterSpacing: "0.36em",
							fontSize: "clamp(12px,1.5vw,64px)",
						}}
					>
						ECONOMIC OS FOR THE AGENT ECONOMY{" "}
						<span style={{ WebkitTextFillColor: "#f13242", color: "#f13242" }}>·</span> CASPER
					</div>
				</motion.div>
				</div>
			</section>
		</>
	);
}
