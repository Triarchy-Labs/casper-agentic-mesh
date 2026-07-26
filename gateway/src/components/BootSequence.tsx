"use client";
import React, { useState, useEffect } from "react";
import Image from "next/image";
import { motion } from "framer-motion";

// Loading lasts exactly 2.5s.
const BOOT_MS = 2500;

const LOG = [
	"◆ casper testnet link ............ ok",
	"◆ escrow a7e6a383… .............. online",
	"◆ rwa oracle feed ............... synced",
	"◆ tribunal bench · 5 models ..... ready",
	"◆ proof-of-liveness ............. mesh healthy",
];

export default function BootSequence({ onComplete }: { onComplete: () => void }) {
	const [progress, setProgress] = useState(0);

	useEffect(() => {
		const start = performance.now();
		let raf = 0;
		const tick = (now: number) => {
			const t = Math.min(1, (now - start) / BOOT_MS);
			setProgress(Math.round(t * 100));
			if (t < 1) {
				raf = requestAnimationFrame(tick);
			} else {
				setTimeout(() => onComplete(), 350); // brief hold at 100 → dissolve
			}
		};
		raf = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(raf);
	}, [onComplete]);

	const shownLogs = Math.min(LOG.length, Math.floor((progress / 100) * (LOG.length + 1)));

	return (
		<motion.div
			// Cinematic push-in toward the "casper" badge, darkening as it dissolves.
			exit={{ opacity: 0, scale: 1.85, filter: "brightness(0.12)" }}
			transition={{ duration: 0.5, ease: [0.7, 0, 0.84, 0] }}
			style={{
				position: "fixed",
				inset: 0,
				zIndex: 99999,
				overflow: "hidden",
				transformOrigin: "50% 40%",
				fontFamily: "ui-monospace, 'Geist Mono', monospace",
				color: "#fff",
			}}
		>
			{/* 4K crafted background — slow ken-burns (auto camera push + drift) */}
			<motion.div
				initial={{ scale: 1, x: "0%", y: "0%" }}
				animate={{ scale: 1.09, x: "-1.1%", y: "-1.4%" }}
				transition={{ duration: 3, ease: "easeOut" }}
				style={{
					position: "absolute",
					inset: 0, // starts at the original framing; scale-up provides its own overscan
					transformOrigin: "center center",
					willChange: "transform",
				}}
			>
                <Image src="/boot.webp" alt="Boot Sequence" fill style={{ objectFit: "cover" }} quality={95} priority={true} />
            </motion.div>
			{/* legibility scrim */}
			<div
				style={{
					position: "absolute",
					inset: 0,
					background:
						"linear-gradient(to bottom, rgba(2,1,3,0.55) 0%, rgba(2,1,3,0.12) 40%, rgba(2,1,3,0.88) 100%)",
				}}
			/>
			{/* soft vignette — depth, darker edges */}
			<div
				style={{
					position: "absolute",
					inset: 0,
					background:
						"radial-gradient(ellipse at 50% 44%, transparent 32%, rgba(0,0,0,0.30) 72%, rgba(0,0,0,0.66) 100%)",
				}}
			/>
			{/* subtle scanlines */}
			<div
				style={{
					position: "absolute",
					inset: 0,
					mixBlendMode: "overlay",
					opacity: 0.06,
					backgroundImage:
						"repeating-linear-gradient(0deg, #000 0 1px, transparent 1px 3px)",
				}}
			/>

			{/* project name — top left */}
			<div
				style={{
					position: "absolute",
					top: "clamp(20px,4vw,48px)",
					left: "clamp(20px,4vw,56px)",
				}}
			>
				<div style={{ fontSize: "clamp(18px,2vw,30px)", fontWeight: 700, letterSpacing: "0.14em" }}>
					TRIARCHY <span style={{ color: "#e03529" }}>//</span> AGENTIC MESH
				</div>
				<div
					style={{
						marginTop: 6,
						fontSize: "clamp(10px,0.9vw,13px)",
						letterSpacing: "0.32em",
						color: "rgba(255,255,255,0.55)",
					}}
				>
					ECONOMIC OS FOR THE AGENT ECONOMY · CASPER
				</div>
			</div>

			{/* system boot log — bottom left, above the bar */}
			<div
				style={{
					position: "absolute",
					left: "clamp(20px,4vw,56px)",
					bottom: "clamp(90px,10vh,150px)",
					display: "flex",
					flexDirection: "column",
					gap: 6,
					fontSize: "clamp(10px,0.85vw,13px)",
					letterSpacing: "0.06em",
					color: "rgba(255,255,255,0.62)",
				}}
			>
				{LOG.slice(0, shownLogs).map((l, i) => (
					<motion.span
						key={i}
						initial={{ opacity: 0, x: -8 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.35 }}
					>
						{l}
					</motion.span>
				))}
			</div>

			{/* big grey semi-transparent number — no % sign */}
			<div
				style={{
					position: "absolute",
					right: "clamp(20px,4vw,64px)",
					bottom: "clamp(64px,9vh,120px)",
					fontSize: "clamp(80px,13vw,220px)",
					fontWeight: 800,
					lineHeight: 0.9,
					color: "rgba(200,200,214,0.30)",
					fontVariantNumeric: "tabular-nums",
					letterSpacing: "-0.04em",
				}}
			>
				{String(progress).padStart(2, "0")}
			</div>

			{/* bottom full-width loading bar — purple track overlaid by pulsing red fill */}
			<div
				style={{
					position: "absolute",
					left: 0,
					right: 0,
					bottom: 0,
					height: "clamp(10px,1.2vh,16px)",
					background: "rgba(224,53,41,0.15)",
					boxShadow:
						"inset 0 0 24px rgba(224,53,41,0.25), 0 -1px 0 rgba(224,53,41,0.4)",
					overflow: "hidden",
				}}
			>
				{/* purple shimmer travelling across the static track (static → animated) */}
				<motion.div
					aria-hidden
					style={{
						position: "absolute",
						inset: 0,
						background:
							"linear-gradient(90deg, transparent, rgba(224,53,41,0.4), transparent)",
					}}
					animate={{ x: ["-100%", "100%"] }}
					transition={{ duration: 1.6, repeat: Infinity, ease: "linear" }}
				/>
				{/* red pulsing fill grows with progress and pulses its glow */}
				<motion.div
					style={{
						position: "absolute",
						top: 0,
						bottom: 0,
						left: 0,
						width: `${progress}%`,
						background: "linear-gradient(90deg, #7a0d1a, #e03529)",
					}}
					animate={{
						boxShadow: [
							"0 0 18px 2px rgba(224,53,41,0.55)",
							"0 0 34px 6px rgba(224,53,41,0.95)",
							"0 0 18px 2px rgba(224,53,41,0.55)",
						],
					}}
					transition={{ duration: 0.9, repeat: Infinity, ease: "easeInOut" }}
				>
					{/* bright leading edge */}
					<div
						style={{
							position: "absolute",
							right: 0,
							top: 0,
							bottom: 0,
							width: 2,
							background: "#fff",
							boxShadow: "0 0 20px 4px rgba(255,255,255,0.8)",
						}}
					/>
				</motion.div>
			</div>
		</motion.div>
	);
}
