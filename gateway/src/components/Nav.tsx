"use client";

import React, { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";
import { useApp } from "@/context/AppContext";

export function Nav() {
	const { connected, pubKey, connecting, showDisconnect, connectWallet, disconnectWallet } = useApp();
	const [menuOpen, setMenuOpen] = useState(false);

	const contentText = connecting ? "CONNECTING…" : connected ? pubKey.toUpperCase() : "WALLET";

	const handleKeyDown = (e: React.KeyboardEvent, action: () => void) => {
		if (e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			action();
		}
	};

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
							pointerEvents: "auto",
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

			<motion.div
				initial={{ opacity: 0, y: -20 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 1.5, ease: [0.16, 1, 0.3, 1] }}
				className="fixed top-0 left-0 right-0 z-[100] flex items-center justify-between px-[5.5vw] pt-[8vh] max-lg:px-[4.10vw] max-lg:pt-[9.59vh] max-sm:px-[5.97vw] max-sm:pt-[11.3vh]"
				style={{
					background: "linear-gradient(to bottom, rgba(0,0,0,0.5) 0%, rgba(0,0,0,0) 100%)",
					pointerEvents: menuOpen ? "auto" : "none",
					fontFamily: "ui-monospace, 'Geist Mono', monospace",
				}}
			>
				<Link href="/" style={{ pointerEvents: "auto", textDecoration: "none", color: "inherit" }}>
					<span style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, letterSpacing: "0.16em", fontSize: "clamp(14px,1.1vw,24px)" }}>
						TRIARCHY{" "}
						<motion.span
							style={{ color: "#e03529", display: "inline-block" }}
							animate={{
								scale: [1, 1.09, 1],
								textShadow: [
									"0 0 0px rgba(224,53,41,0)",
									"0 0 11px rgba(224,53,41,0.8)",
									"0 0 0px rgba(224,53,41,0)",
								],
							}}
							transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut" }}
						>
							/
						</motion.span>
						<motion.span
							style={{ color: "#e03529", display: "inline-block" }}
							animate={{
								scale: [1, 1.09, 1],
								textShadow: [
									"0 0 0px rgba(224,53,41,0)",
									"0 0 11px rgba(224,53,41,0.8)",
									"0 0 0px rgba(224,53,41,0)",
								],
							}}
							transition={{ duration: 4.8, repeat: Infinity, ease: "easeInOut", delay: 2.4 }}
						>
							/
						</motion.span>{" "}
						MESH
					</span>
				</Link>

				<nav style={{ pointerEvents: "auto" }} className="flex items-center gap-[4.1vw]">
					<div className="flex items-center gap-[4.1vw] max-lg:hidden">
						<BracketLink label="BOUNTIES" href="/bounties" />
						<BracketLink label="DASHBOARD" href="/dashboard" />
					</div>
					<div className="flex items-center gap-[4.1vw] relative">
						<div
							role="button"
							tabIndex={0}
							onClick={connectWallet}
							onKeyDown={(e) => handleKeyDown(e, connectWallet)}
							className="min-w-[140px] text-center inline-block"
						>
							<BracketLink label={contentText} />
						</div>

						{/* Disconnect Bubble */}
						<motion.div
							role="button"
							tabIndex={0}
							initial={{ opacity: 0, y: -10, pointerEvents: "none" }}
							animate={{ opacity: showDisconnect ? 1 : 0, y: showDisconnect ? 10 : -10, pointerEvents: showDisconnect ? "auto" : "none" }}
							style={{
								position: "absolute",
								top: "100%",
								right: 0,
								background: "var(--red-100)",
								border: "1px solid var(--red-700)",
								padding: "8px 16px",
								borderRadius: "6px",
								cursor: "pointer",
								color: "var(--red-700)",
								marginTop: "8px"
							}}
							onClick={disconnectWallet}
							onKeyDown={(e) => handleKeyDown(e, disconnectWallet)}
						>
							<span className="label-14-mono">DISCONNECT</span>
						</motion.div>

						<div
							role="button"
							tabIndex={0}
							onClick={() => setMenuOpen(!menuOpen)}
							onKeyDown={(e) => handleKeyDown(e, () => setMenuOpen(!menuOpen))}
							className="z-[100]"
						>
							<BracketLink label={menuOpen ? "CLOSE" : "MENU"} />
						</div>
					</div>
				</nav>
			</motion.div>
		</>
	);
}

function BracketLink({ label, href }: { label: string; href?: string }) {
	const content = (
		<motion.span
			className="bracket-link relative uppercase tracking-[0.18em] text-[14px] xl:text-[0.97vw] font-extralight flex gap-[0.5em]"
			initial="rest"
			whileHover="hover"
			animate="rest"
		>
			<motion.span
				variants={{
					rest: { opacity: 0.3, x: 0 },
					hover: { opacity: 1, x: -4, color: "#e03529" },
				}}
			>
				[
			</motion.span>
			<motion.span
				variants={{
					rest: { color: "rgba(255,255,255,0.7)" },
					hover: { color: "rgba(255,255,255,1)", textShadow: "0 0 8px rgba(255,255,255,0.4)" },
				}}
			>
				{label}
			</motion.span>
			<motion.span
				variants={{
					rest: { opacity: 0.3, x: 0 },
					hover: { opacity: 1, x: 4, color: "#e03529" },
				}}
			>
				]
			</motion.span>
		</motion.span>
	);

	if (href) {
		return (
			<Link href={href} style={{ textDecoration: "none" }}>
				{content}
			</Link>
		);
	}
	return <span style={{ cursor: "pointer" }}>{content}</span>;
}
