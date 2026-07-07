"use client";
import React, { useEffect, useState } from "react";
import { motion, useScroll, useTransform, useSpring, useMotionValueEvent } from "framer-motion";

// produx's signature scroll ease + scrub:1.5 smoothing, extracted from their bundle.
const PX_EASE = [0.2, 0.6, 0.35, 1] as const;

// Animated bracketed nav item — brackets spread out on hover.
function BracketLink({ label, href = "#" }: { label: string; href?: string }) {
	const [hovered, setHovered] = useState(false);
	return (
		<a
			href={href}
			onMouseEnter={() => setHovered(true)}
			onMouseLeave={() => setHovered(false)}
			style={{
				display: "inline-flex",
				alignItems: "center",
				gap: 6,
				fontFamily: "ui-monospace, 'Geist Mono', monospace",
				fontSize: "clamp(12px,1.15vw,19px)",
				letterSpacing: "0.18em",
				color: hovered ? "#fff" : "rgba(255,255,255,0.66)",
				textDecoration: "none",
				transition: "color 0.25s ease",
				whiteSpace: "nowrap",
			}}
		>
			<motion.span
				animate={{ x: hovered ? -4 : 0, color: hovered ? "#f13242" : "rgba(255,255,255,0.4)" }}
				transition={{ duration: 0.4, ease: PX_EASE }}
			>
				[
			</motion.span>
			{label}
			<motion.span
				animate={{ x: hovered ? 4 : 0, color: hovered ? "#f13242" : "rgba(255,255,255,0.4)" }}
				transition={{ duration: 0.4, ease: PX_EASE }}
			>
				]
			</motion.span>
		</a>
	);
}

export function ScrollHero() {
	const { scrollY } = useScroll();
	// Emulate ScrollTrigger scrub:1.5 — a soft lag behind the raw scroll so the
	// wordmark eases into the nav instead of tracking 1:1 (their cinematic feel).
	const smoothY = useSpring(scrollY, { stiffness: 55, damping: 18, restDelta: 0.5 });
	const [scrolled, setScrolled] = useState(false);
	const [mouse, setMouse] = useState({ x: -200, y: -200 });

	useMotionValueEvent(scrollY, "change", (v) => setScrolled(v > 14));

	useEffect(() => {
		const onMove = (e: MouseEvent) => setMouse({ x: e.clientX, y: e.clientY });
		window.addEventListener("mousemove", onMove);
		return () => window.removeEventListener("mousemove", onMove);
	}, []);

	// Giant wordmark shrinks + rises + fades over the first ~520px of scroll.
	const wmScale = useTransform(smoothY, [0, 520], [1, 0.2]);
	const wmY = useTransform(smoothY, [0, 520], [0, -140]);
	const wmOpacity = useTransform(smoothY, [0, 340, 520], [1, 0.2, 0]);
	const blur = useTransform(smoothY, [300, 520], [0, 9]);
	const wmFilter = useTransform(blur, (b) => `blur(${b}px)`);

	// Docked bracket nav fades in as the wordmark shrinks away.
	const navOpacity = useTransform(smoothY, [140, 360], [0, 1]);
	const navY = useTransform(smoothY, [140, 360], [-18, 0]);

	return (
		<>
			{/* cursor-follow [ SCROLL DOWN ] label — vanishes on first scroll */}
			<motion.div
				aria-hidden
				animate={{ opacity: scrolled ? 0 : 0.9 }}
				transition={{ duration: 0.3 }}
				style={{
					position: "fixed",
					left: mouse.x + 22,
					top: mouse.y + 8,
					zIndex: 60,
					pointerEvents: "none",
					fontFamily: "ui-monospace, 'Geist Mono', monospace",
					fontSize: 11,
					letterSpacing: "0.24em",
					color: "rgba(255,255,255,0.8)",
					whiteSpace: "nowrap",
					textShadow: "0 0 10px rgba(0,0,0,0.6)",
				}}
			>
				<span style={{ color: "#f13242" }}>[</span> scroll down{" "}
				<span style={{ color: "#f13242" }}>]</span>
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
					backdropFilter: "blur(10px)",
					background: "linear-gradient(to bottom, rgba(4,2,3,0.6), transparent)",
				}}
			>
				<span style={{ fontWeight: 700, letterSpacing: "0.16em", fontSize: "clamp(14px,1.3vw,22px)" }}>
					TRIARCHY <span style={{ color: "#f13242" }}>//</span> MESH
				</span>
				<nav style={{ display: "flex", gap: "clamp(14px,2.4vw,40px)", alignItems: "center" }}>
					<BracketLink label="BOUNTIES" href="/bounties" />
					<BracketLink label="DASHBOARD" href="/dashboard" />
					<BracketLink label="CONNECT WALLET" />
				</nav>
			</motion.header>

			{/* giant scroll-shrink wordmark — first screen */}
			<section
				style={{
					minHeight: "100vh",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					position: "relative",
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
							fontFamily: "'Sora', ui-sans-serif, sans-serif",
							fontWeight: 800,
							fontSize: "clamp(72px, 11.9vw, 640px)", // produx hero = 11.9vw
							lineHeight: 0.88,
							letterSpacing: "-0.035em",
							color: "#fff",
							textShadow: "0 8px 60px rgba(0,0,0,0.6)",
						}}
					>
						TRIARCHY
					</div>
					<div
						style={{
							marginTop: "1.4rem",
							fontFamily: "ui-monospace, 'Geist Mono', monospace",
							letterSpacing: "0.36em",
							fontSize: "clamp(12px,1.5vw,26px)",
							color: "rgba(255,255,255,0.6)",
						}}
					>
						ECONOMIC OS FOR THE AGENT ECONOMY{" "}
						<span style={{ color: "#f13242" }}>·</span> CASPER
					</div>
				</motion.div>

				{/* bottom hint bar echoing the loader */}
				<div
					style={{
						position: "absolute",
						bottom: 0,
						left: 0,
						right: 0,
						height: 3,
						background:
							"linear-gradient(90deg, transparent, rgba(168,85,247,0.5) 30%, #f13242 60%, transparent)",
						opacity: 0.7,
					}}
				/>
			</section>
		</>
	);
}
