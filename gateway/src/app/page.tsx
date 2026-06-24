"use client";
import React, { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";

const FONT_HEADING = "'Helvetica Now Display', 'Inter', sans-serif";
const FONT_BODY = "'Inter', 'DM Sans', sans-serif";

// ============================================================
// LUSION HOVER SYSTEM:
// Default = Monochrome premium (white-on-dark)
// Hover  = Original green (#00ff41) bleeds in as "reward"
// ============================================================
const palette = {
	dark: {
		accent: "rgba(255,255,255,0.9)",
		accentHover: "#00ff41",               // 🟢 LUSION: green reveal
		accentMuted: "rgba(255,255,255,0.5)",
		border: "rgba(255,255,255,0.12)",
		borderHover: "rgba(0,255,65,0.4)",    // 🟢
		glassBg: "rgba(20,20,28,0.45)",
		glassBgHover: "rgba(0,15,0,0.5)",     // 🟢
		glow: "rgba(255,255,255,0.06)",
		glowHover: "rgba(0,255,65,0.25)",     // 🟢
		text: "#fff",
		textMuted: "#888",
		btnBg: "#fff",
		btnBgHover: "#00ff41",                // 🟢
		btnText: "#0a0a0a",
	},
	light: {
		accent: "#fff",
		accentHover: "#00ff41",
		accentMuted: "rgba(255,255,255,0.5)",
		border: "rgba(255,255,255,0.12)",
		borderHover: "rgba(0,255,65,0.4)",
		glassBg: "rgba(10,10,10,0.85)",
		glassBgHover: "rgba(0,15,0,0.9)",
		glow: "rgba(255,255,255,0.06)",
		glowHover: "rgba(0,255,65,0.25)",
		text: "#fff",
		textMuted: "#aaa",
		btnBg: "#fff",
		btnBgHover: "#00ff41",
		btnText: "#0a0a0a",
	},
};

// CSS transition for all Lusion hover elements
const lusionTransition = "all 0.35s cubic-bezier(0.4, 0, 0.2, 1)";


function FloatingConnector({ theme }: { theme: "dark" | "light" }) {
	const p = palette[theme];
	return (
		<motion.div
			initial={{ y: 50, opacity: 0 }}
			whileInView={{ y: 0, opacity: 1 }}
			viewport={{ once: true }}
			transition={{ delay: 0.2, type: "spring", stiffness: 100 }}
			style={{
				position: "relative",
				margin: "4rem auto 8rem auto",
				width: "fit-content",
				display: "flex",
				flexWrap: "wrap",
				justifyContent: "center",
				gap: "24px",
				alignItems: "center",
			}}
		>
			<motion.button
				whileHover={{ scale: 1.05 }}
				whileTap={{ scale: 0.95 }}
				onClick={() => window.location.href = "/dashboard"}
				style={{
					background: p.accentHover,
					color: "#000",
					border: `1px solid ${p.accentHover}`,
					padding: "16px 48px",
					borderRadius: "40px",
					cursor: "pointer",
					fontFamily: FONT_HEADING,
					fontSize: "0.9rem",
					fontWeight: 600,
					letterSpacing: "0.15em",
					boxShadow: `0 0 40px ${p.glowHover}`,
					transition: lusionTransition,
				}}
			>
				LAUNCH BOUNTY
			</motion.button>

			<motion.button
				whileHover={{ scale: 1.05, background: theme === "dark" ? "rgba(255,255,255,0.1)" : "rgba(0,0,0,0.1)" }}
				whileTap={{ scale: 0.95 }}
				onClick={() => document.getElementById("agent-registry")?.scrollIntoView({ behavior: "smooth" })}
				style={{
					background: "transparent",
					color: p.text,
					border: `1px solid rgba(255,255,255,0.2)`,
					padding: "16px 48px",
					borderRadius: "40px",
					cursor: "pointer",
					fontFamily: FONT_HEADING,
					fontSize: "0.9rem",
					fontWeight: 600,
					backdropFilter: "blur(12px)",
					WebkitBackdropFilter: "blur(12px)",
					letterSpacing: "0.15em",
					transition: lusionTransition,
				}}
			>
				DEPLOY AGENT
			</motion.button>
		</motion.div>
	);
}



function ScrollToTop({ theme }: { theme: "dark" | "light" }) {
	const p = palette[theme];
	const [hovered, setHovered] = useState(false);

	return (
		<motion.div
			initial={{ opacity: 0, y: 100 }}
			whileInView={{ opacity: 1, y: 0 }}
			viewport={{ once: false, amount: 0.1 }}
			transition={{ duration: 1.4, ease: [0.16, 1, 0.3, 1] }}
			style={{
				display: "flex",
				justifyContent: "center",
				marginTop: "4rem",
				paddingBottom: "1rem",
				zIndex: 100,
			}}
		>
			<div className="micro-drift">
				<motion.button
					className="rare-snake-border"
					onMouseEnter={() => setHovered(true)}
					onMouseLeave={() => setHovered(false)}
					onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
					style={{
						position: "relative",
						width: 56,
						height: 56,
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						background: hovered ? p.glassBgHover : p.glassBg,
						border: `1px solid ${hovered ? p.borderHover : p.border}`,
						color: hovered ? p.accentHover : p.accent,
						backdropFilter: "blur(40px) saturate(2)",
						WebkitBackdropFilter: "blur(40px) saturate(2)",
						cursor: "pointer",
						borderRadius: "50%",
						fontSize: "1.2rem",
						fontFamily: "'SF Mono', monospace",
						boxShadow: hovered 
							? `0 0 50px ${p.glowHover}, inset 0 0 20px rgba(0,255,65,0.3)` 
							: `0 20px 40px rgba(0,0,0,0.6), inset 0 0 15px rgba(255,255,255,0.05)`,
						transition: "all 0.6s cubic-bezier(0.1, 0.8, 0.2, 1)",
						transform: `scale(${hovered ? 1.15 : 1})`,
						overflow: "hidden",
					}}
					aria-label="Scroll to top"
				>
					{/* Inner ambient glow layer */}
					<div style={{
						position: "absolute",
						top: "10%", left: "10%", right: "10%", bottom: "10%",
						background: hovered ? "radial-gradient(circle, rgba(0,255,65,0.4) 0%, transparent 70%)" : "radial-gradient(circle, rgba(255,255,255,0.1) 0%, transparent 70%)",
						filter: "blur(10px)",
						borderRadius: "50%",
						zIndex: 0,
						transition: "all 0.6s ease",
					}} />
					
					<span style={{ position: "relative", zIndex: 1, filter: hovered ? "drop-shadow(0 0 8px rgba(0,255,65,0.8))" : "none" }}>
						↑
					</span>
				</motion.button>
			</div>
		</motion.div>
	);
}

import HollywoodTelemetry from "@/components/HollywoodTelemetry";
import BootSequence from "@/components/BootSequence";
import GlitchText from "@/components/GlitchText";
import AgentNetworkGrid from "@/components/AgentNetworkGrid";
import CustomCursor from "@/components/CustomCursor";
import { AnimatedArchitecture } from "@/components/AnimatedArchitecture";
import FloatingModules from "@/components/FloatingModules";
import { SystemWidget } from "@/components/SystemWidget";
import { Nav } from "@/components/Nav";


export default function Page() {
	const [booted, setBooted] = useState(false);
	const [theme, setTheme] = useState<"dark" | "light">("dark");
	const p = palette[theme];

	// Lusion postInvert: sync theme to html data-theme for global CSS canvas inversion
	React.useEffect(() => {
		document.documentElement.setAttribute("data-theme", theme);
		document.body.style.backgroundColor = theme === "dark" ? "#000" : "#fff";
	}, [theme]);

	// Lusion hover states for inline elements
	const [hoverTheme, setHoverTheme] = useState(false);
	const [hoverGithub, setHoverGithub] = useState(false);
	const [hoverCTA, setHoverCTA] = useState(false);
	const [hover3d, setHover3d] = useState(false);

	// Magnetic cursor offsets
	const [offsetTheme, setOffsetTheme] = useState({ x: 0, y: 0 });
	const [offsetGithub, setOffsetGithub] = useState({ x: 0, y: 0 });
	const themeRef = React.useRef<HTMLButtonElement>(null);
	const githubRef = React.useRef<HTMLAnchorElement>(null);
	const magneticMove = (ref: React.RefObject<HTMLElement | null>, e: React.MouseEvent, setFn: (v: {x:number,y:number}) => void) => {
		if (!ref.current) return;
		const r = ref.current.getBoundingClientRect();
		setFn({ x: (e.clientX - r.left - r.width/2) * 0.3, y: (e.clientY - r.top - r.height/2) * 0.3 });
	};

	return (
		<main
			style={{ minHeight: "100vh", position: "relative", overflowX: "hidden", background: "transparent" }}
		>
			<AnimatePresence>
				{!booted && <BootSequence key="boot" onComplete={() => setBooted(true)} />}
			</AnimatePresence>

			{booted && (
				<>
					<CustomCursor />
					<SystemWidget />
					<Nav />

					{/* Theme Switcher — Lusion hover */}
					<motion.button
						ref={themeRef}
						onClick={() => setTheme((t) => (t === "dark" ? "light" : "dark"))}
						initial={{ opacity: 0, y: -20 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 1, duration: 1 }}
						onMouseEnter={() => setHoverTheme(true)}
						onMouseMove={(e) => magneticMove(themeRef, e, setOffsetTheme)}
						onMouseLeave={() => { setHoverTheme(false); setOffsetTheme({x:0,y:0}); }}
						style={{
							position: "fixed",
							bottom: "var(--pos-fab-bottom)",
							right: "var(--pos-fab-right)",
							zIndex: 100,
							padding: "10px 24px",
							background: hoverTheme ? p.glassBgHover : p.glassBg,
							border: `1px solid ${hoverTheme ? p.borderHover : p.border}`,
							color: hoverTheme ? p.accentHover : p.accent,
							backdropFilter: "blur(15px)",
							cursor: "pointer",
							fontFamily: FONT_HEADING,
							borderRadius: "30px",
							fontWeight: 500,
							letterSpacing: "0.1em",
							boxShadow: hoverTheme ? `0 0 25px ${p.glowHover}` : "none",
							transition: lusionTransition,
							transform: `translate(${offsetTheme.x}px, ${offsetTheme.y}px)`,
						}}
					>
						{/* Invisible proximity hitbox */ }
						<div style={{ position: "absolute", top: -40, left: -40, right: -40, bottom: -40, zIndex: -1 }} />
						{theme === "dark" ? "◉ DARK" : "◌ LIGHT"}
					</motion.button>
					
					{/* GitHub — Lusion hover + blur */}
					<motion.a
						ref={githubRef}
						href="https://github.com/Triarchy-Labs"
						target="_blank"
						rel="noopener noreferrer"
						initial={{ opacity: 0, scale: 0.8 }}
						animate={{ opacity: 1, scale: 1 }}
						transition={{ delay: 1.2, duration: 1 }}
						onMouseEnter={() => setHoverGithub(true)}
						onMouseMove={(e) => magneticMove(githubRef, e, setOffsetGithub)}
						onMouseLeave={() => { setHoverGithub(false); setOffsetGithub({x:0,y:0}); }}
						style={{
							position: "fixed",
							bottom: "var(--pos-fab-bottom)",
							right: "calc(var(--pos-fab-right) + 130px)",
							zIndex: 100,
							width: 44,
							height: 44,
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							background: hoverGithub ? p.glassBgHover : p.glassBg,
							border: `1px solid ${hoverGithub ? p.borderHover : p.border}`,
							color: hoverGithub ? p.accentHover : p.accent,
							backdropFilter: "blur(15px)",
							cursor: "pointer",
							borderRadius: "50%",
							boxShadow: hoverGithub ? `0 0 25px ${p.glowHover}` : `0 8px 24px ${p.glow}`,
							transition: lusionTransition,
							transform: `translate(${offsetGithub.x}px, ${offsetGithub.y}px) ${hoverGithub ? "scale(1.1) rotate(5deg)" : "scale(1)"}`,
						}}
					>
						{/* Invisible proximity hitbox */ }
						<div style={{ position: "absolute", top: -40, left: -40, right: -40, bottom: -40, zIndex: -1 }} />
						<svg height="24" width="24" viewBox="0 0 16 16" fill="currentColor">
							<path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path>
						</svg>
					</motion.a>

					<div style={{ position: "relative", minHeight: "100vh", width: "100%" }}>

						<motion.div
							initial={{ opacity: 0, y: 50 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 1.5, ease: "easeOut" }}
							style={{
								position: "relative",
								zIndex: 10,
								display: "flex",
								flexDirection: "column",
								alignItems: "center",
								justifyContent: "center",
								paddingTop: "15vh",
								minHeight: "100vh",
								padding: "2rem",
								paddingBottom: "15rem"
							}}
						>
							<GlitchText text="SOVEREIGN GATEWAY" theme={theme} />
							
							<motion.p
								initial={{ opacity: 0 }}
								animate={{ opacity: 1 }}
								transition={{ delay: 1.5, duration: 1 }}
								style={{
									color: p.accentMuted,
									fontFamily: FONT_HEADING,
									fontSize: "1.2rem",
									marginTop: "2rem",
									letterSpacing: "0.4em",
									textTransform: "uppercase",
									fontWeight: 400,
								}}
							>
								x402 protocol / Sovereign Matrix
							</motion.p>

							{/* MAIN CTA — Gateway to Bounty Board */}
							<motion.button
								whileTap={{ scale: 0.95 }}
								onMouseEnter={() => setHoverCTA(true)}
								onMouseLeave={() => setHoverCTA(false)}
								onClick={() => window.location.href = "/bounties"}
								className="snake-border"
								style={{
									marginTop: "4rem",
									padding: "1.5rem 4rem",
									border: `1px solid ${hoverCTA ? p.borderHover : p.border}`,
									borderRadius: "12px",
									background: hoverCTA ? p.glassBgHover : p.glassBg,
									backdropFilter: "blur(24px) saturate(1.2)",
									WebkitBackdropFilter: "blur(24px) saturate(1.2)",
									cursor: "pointer",
									position: "relative",
									overflow: "hidden",
									boxShadow: hoverCTA ? `0 0 40px ${p.glowHover}` : "none",
									transition: lusionTransition,
									transform: hoverCTA ? "scale(1.02)" : "scale(1)",
								}}
							>
								<div
									style={{
										position: "absolute",
										top: 0,
										left: 0,
										right: 0,
										height: "1px",
										background: `linear-gradient(90deg, transparent, ${hoverCTA ? p.accentHover : p.border}, transparent)`,
										transition: lusionTransition,
									}}
								/>
								<div
									style={{
										position: "absolute",
										bottom: 0,
										left: 0,
										right: 0,
										height: "1px",
										background: `linear-gradient(90deg, transparent, ${hoverCTA ? p.accentHover : p.border}, transparent)`,
										transition: lusionTransition,
									}}
								/>
								<p
									style={{
										color: hoverCTA ? p.accentHover : p.text,
										fontFamily: FONT_HEADING,
										textAlign: "center",
										margin: 0,
										fontWeight: 500,
										letterSpacing: "0.15em",
										fontSize: "1.1rem",
										transition: lusionTransition,
										textShadow: hoverCTA ? `0 0 15px ${p.glowHover}` : "none",
									}}
								>
									[ ACCESS BOUNTY BOARD ]
								</p>
							</motion.button>

							{/* Live stats row — green sweep animation */}
							<motion.div
								initial={{ opacity: 0 }}
								animate={{ opacity: 1 }}
								transition={{ delay: 2, duration: 1 }}
								className="green-sweep-stats"
								style={{
									marginTop: "3rem",
									display: "flex",
									gap: "3rem",
									justifyContent: "center",
									fontFamily: "'SF Mono', monospace",
									fontSize: "0.75rem",
									letterSpacing: "0.1em",
								}}
							>
								<span>NODES: 6</span>
								<span>⬡</span>
								<span>LATENCY: 12ms</span>
								<span>⬡</span>
								<span>UPTIME: 99.97%</span>
							</motion.div>
						</motion.div>

						<motion.div
							initial={{ opacity: 0, x: -50 }}
							animate={{ opacity: 1, x: 0 }}
							transition={{ delay: 2, duration: 1 }}
						>
							<HollywoodTelemetry theme={theme} />
						</motion.div>
					</div>

					{/* SYSTEM ARCHITECTURE */}
					<section style={{ position: "relative", zIndex: 10, width: "100%", padding: "4rem 0" }}>
						<motion.h2 
							initial={{ opacity: 0, y: 20 }}
							whileInView={{ opacity: 1, y: 0 }}
							viewport={{ once: true }}
							style={{ 
								fontFamily: FONT_HEADING, 
								fontSize: "2rem", 
								textAlign: "center", 
								borderBottom: "none", 
								paddingBottom: "1rem", 
								marginBottom: "2rem", 
								color: p.text,
								fontWeight: 500,
								letterSpacing: "0.1em",
							}}
						>
							SYSTEM ARCHITECTURE
						</motion.h2>

						<div
							style={{ position: "relative" }}
							onMouseEnter={() => setHover3d(true)}
							onMouseLeave={() => setHover3d(false)}
						>
							<AnimatedArchitecture theme={theme} />
							<FloatingModules hovered={hover3d} />
						</div>
						<FloatingConnector theme={theme} />
					</section>

					{/* AGENT REGISTRY */}
					<div id="agent-registry">
						<AgentNetworkGrid theme={theme} />
					</div>

					{/* FOOTER — Lusion hover on CTA */}
					<footer style={{ 
						position: "relative", 
						zIndex: 10, 
						padding: "6rem 2rem 4rem", 
						marginTop: "4rem", 
						background: theme === "dark" 
							? "linear-gradient(to bottom, rgba(255,255,255,0.03), rgba(255,255,255,0.06))" 
							: "linear-gradient(to bottom, rgba(255,255,255,0.35), rgba(255,255,255,0.5))",
						backdropFilter: "blur(24px) saturate(1.2)",
						WebkitBackdropFilter: "blur(24px) saturate(1.2)",
						borderRadius: "16px 16px 0 0",
						border: `1px solid ${theme === "dark" ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.06)"}`,
						borderBottom: "none",
					}}>
						{/* Green glow divider at top */}
						<div style={{
							position: "absolute",
							top: 0,
							left: "20%",
							right: "20%",
							height: "1px",
							background: `linear-gradient(90deg, transparent, ${theme === "dark" ? "rgba(0,255,65,0.3)" : "rgba(0,100,34,0.2)"}, transparent)`,
						}} />
						<div style={{ maxWidth: 1200, margin: "0 auto", textAlign: "center" }}>
							<h2 style={{ 
								fontFamily: FONT_HEADING, 
								fontSize: "2.5rem", 
								color: p.text, 
								marginBottom: "2rem", 
								letterSpacing: "0.08em",
								fontWeight: 500,
							}}>
								THE MESH MANIFESTO
							</h2>
							<p style={{ 
								fontFamily: FONT_BODY, 
								fontSize: "1.2rem", 
								lineHeight: 1.8, 
								color: p.textMuted, 
								maxWidth: 800, 
								margin: "0 auto 4rem" 
							}}>
								Other solutions build single, isolated AI agents holding private keys, blindly trusting inputs, and bleeding liquidity. We built the <strong>Immune System</strong> for the entire AI economy. By becoming a Sovereign Integrator, you align your nodes with zero-trust execution and global liquidity flow.
							</p>
							
							<ContactModal theme={theme} />
							
							{/* Copyright with green sweep animation */}
							<div 
								className="green-sweep-text"
								style={{ 
									marginTop: "6rem", 
									fontFamily: FONT_BODY,
									fontSize: "0.85rem",
									letterSpacing: "0.05em",
								}}
							>
								© {new Date().getFullYear()} Triarchy Labs // x402 Global Routing Grid
							</div>
							
							{/* Scroll To Top localized to footer */}
							<ScrollToTop theme={theme} />
						</div>

					{/* Infinity glow at bottom — thin bright line */}
					<div style={{
						position: "absolute",
						bottom: -2,
						left: "3%",
						right: "3%",
						height: "10px",
						background: `radial-gradient(ellipse at center bottom, ${theme === "dark" ? "rgba(0,255,65,0.35)" : "rgba(0,100,34,0.2)"}, transparent 80%)`,
						filter: "blur(6px)",
						pointerEvents: "none",
						animation: "infinityPulse 4s ease-in-out infinite",
					}} />
					</footer>

					{/* Scroll to top removed from global fixed scope */}
				</>
			)}
		</main>
	);
}

function ContactModal({ theme }: { theme: "dark" | "light" }) {
	const p = palette[theme];
	const [isOpen, setIsOpen] = useState(false);
	const [hovered, setHovered] = useState(false);

	return (
		<div style={{ display: "inline-block", position: "relative" }}>
			<AnimatePresence>
				{isOpen && (
					<motion.div
						initial={{ opacity: 0, y: 30, scale: 0.95 }}
						animate={{ opacity: 1, y: 0, scale: 1 }}
						exit={{ opacity: 0, y: 20, scale: 0.95 }}
						transition={{ type: "spring", stiffness: 300, damping: 25 }}
						style={{
							position: "absolute",
							bottom: "140%",
							left: "50%",
							transform: "translateX(-50%)",
							width: "680px",
							maxWidth: "90vw",
							background: theme === "dark" ? "#070707" : "#fff",
							borderRadius: "24px",
							padding: "2.5rem 3rem",
							boxShadow: theme === "dark" 
								? "0 40px 80px rgba(0,0,0,0.9), 0 0 0 1px rgba(255,255,255,0.08)" 
								: "0 40px 80px rgba(0,0,0,0.2), 0 0 0 1px rgba(0,0,0,0.05)",
							zIndex: 100,
							textAlign: "left"
						}}
					>
						<div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: "2rem" }}>
							<div style={{ display: "flex", alignItems: "baseline", gap: "1rem" }}>
								<h3 style={{ margin: 0, color: p.text, fontFamily: FONT_HEADING, fontSize: "1.2rem", letterSpacing: "0.1em", fontWeight: 600 }}>
									DIRECT TRANSMISSION
								</h3>
								<span style={{ color: p.textMuted, fontSize: "0.75rem", fontFamily: FONT_BODY, letterSpacing: "0.05em" }}>
									(SECURE END-TO-END ENCRYPTED)
								</span>
							</div>
							<button onClick={() => setIsOpen(false)} style={{ background: "transparent", border: "none", color: p.textMuted, cursor: "pointer", fontSize: "1.4rem", transition: "color 0.2s" }} onMouseEnter={(e) => e.currentTarget.style.color = p.text} onMouseLeave={(e) => e.currentTarget.style.color = p.textMuted}>✕</button>
						</div>

						{/* Horizontal form layout layout */}
						<div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: "1rem" }}>
							<div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
								<input 
									type="email" 
									placeholder="YOUR_TRANSPONDER_EMAIL" 
									style={{ width: "100%", padding: "16px 20px", background: theme === "dark" ? "rgba(255,255,255,0.05)" : "rgba(0,0,0,0.05)", border: `1px solid transparent`, borderRadius: "12px", color: p.text, fontFamily: FONT_BODY, fontSize: "0.85rem", outline: "none", letterSpacing: "0.05em", transition: "border 0.3s" }} 
									onFocus={(e) => e.currentTarget.style.border = `1px solid ${p.borderHover}`}
									onBlur={(e) => e.currentTarget.style.border = `1px solid transparent`}
								/>
								<button 
									style={{ width: "100%", height: "100%", minHeight: "50px", borderRadius: "12px", background: p.text, border: "none", color: theme === "dark" ? "#000" : "#fff", display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0 20px", cursor: "pointer", fontFamily: FONT_HEADING, fontSize: "0.85rem", fontWeight: 600, letterSpacing: "0.1em", transition: "transform 0.2s" }} 
									onMouseEnter={(e) => e.currentTarget.style.transform = "scale(1.02)"} 
									onMouseLeave={(e) => e.currentTarget.style.transform = "scale(1)"}
								>
									<span>TRANSMIT</span>
									<span style={{ fontSize: "1.2rem", fontWeight: "bold" }}>→</span>
								</button>
							</div>
							
							<textarea 
								placeholder="Input transmission payload... (Integration requests, alliance queries)" 
								rows={4}
								style={{ width: "100%", padding: "16px 20px", background: theme === "dark" ? "rgba(255,255,255,0.05)" : "rgba(0,0,0,0.02)", border: `1px solid transparent`, borderRadius: "12px", color: p.text, fontFamily: FONT_BODY, fontSize: "0.85rem", outline: "none", resize: "none", transition: "border 0.3s" }} 
								onFocus={(e) => e.currentTarget.style.border = `1px solid ${p.borderHover}`}
								onBlur={(e) => e.currentTarget.style.border = `1px solid transparent`}
							/>
						</div>
					</motion.div>
				)}
			</AnimatePresence>

			{!isOpen && (
				<motion.button
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					whileTap={{ scale: 0.95 }}
					onClick={() => setIsOpen(true)}
					onMouseEnter={() => setHovered(true)}
					onMouseLeave={() => setHovered(false)}
					style={{
						display: "inline-block",
						padding: "1rem 3rem",
						background: hovered ? p.btnBgHover : "transparent",
						color: hovered ? "#000" : p.text,
						fontFamily: FONT_HEADING,
						fontWeight: 500,
						border: `1px solid ${hovered ? p.btnBgHover : p.border}`,
						borderRadius: "40px",
						letterSpacing: "0.1em",
						boxShadow: hovered ? `0 0 40px ${p.glowHover}` : "none",
						transition: lusionTransition,
						transform: hovered ? "scale(1.05)" : "scale(1)",
						cursor: "pointer"
					}}
				>
					[ CONTACT THE MESH ]
				</motion.button>
			)}
		</div>
	);
}
