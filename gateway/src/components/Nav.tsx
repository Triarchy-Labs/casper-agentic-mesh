"use client";

import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";

// Genuine Casper Wallet Provider Integration (Zero-Mock Policy)
const checkCasperConnected = async () => {
    if (typeof window !== "undefined" && window.casperWallet) {
        try {
            const isConnected = await window.casperWallet.isConnected();
            return { isConnected };
        } catch {
            return { isConnected: false };
        }
    }
    return { isConnected: false };
};

const requestAccess = async () => {
    if (typeof window !== "undefined" && window.casperWallet) {
        try {
            await window.casperWallet.requestConnection();
            const activeKey = await window.casperWallet.getActivePublicKey();
            return { address: activeKey };
        } catch (error) {
            return { error };
        }
    }
    return { error: "Casper Wallet not installed" };
};

export function Nav() {
	const [connected, setConnected] = useState(false);
	const [pubKey, setPubKey] = useState("");
	const [connecting, setConnecting] = useState(false);
	const [walletMissing, setWalletMissing] = useState(false);
    const [hoverLogo, setHoverLogo] = useState(false);
    const [showDisconnect, setShowDisconnect] = useState(false);
    const [menuOpen, setMenuOpen] = useState(false);

    useEffect(() => {
		const checkConn = async () => {
			try {
				const status = await checkCasperConnected();
				if (status.isConnected) {
					const access = await requestAccess();
					if (access.address) {
						const key = access.address;
						setPubKey(key.substring(0, 4) + "..." + key.substring(key.length - 4));
						setConnected(true);
					}
				}
			} catch (e) {
				console.warn("[Nav] Casper wallet connection check failed:", e);
			}
		};
		checkConn();
	}, []);

	const handleConnect = async () => {
        if (connected) {
            setShowDisconnect(!showDisconnect);
            return;
        }

		if (walletMissing) {
			window.open("https://www.casperwallet.io/", "_blank");
			return;
		}

		setConnecting(true);
		try {
            if (typeof window === "undefined" || !window.casperWallet) {
                setWalletMissing(true);
                return;
            }

            const access = await requestAccess();
            if (access.error) {
                console.log("Casper Wallet connection rejected.", access.error);
            } else if (access.address) {
                const key = access.address;
                setPubKey(key.substring(0, 4) + "..." + key.substring(key.length - 4));
                setConnected(true);
            }
		} catch (error) {
			console.error("Casper Wallet connect failed", error);
		} finally {
			setConnecting(false);
		}
	};

    const handleDisconnect = () => {
        setConnected(false);
        setPubKey("");
        setShowDisconnect(false);
    };

	const contentText = connecting ? "CONNECTING…" : connected ? pubKey.toUpperCase() : walletMissing ? "GET CASPER WALLET" : "CONNECT WALLET";

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
				</Link>

				<nav style={{ pointerEvents: "auto" }} className="flex items-center gap-[4.1vw]">
					<div className="flex items-center gap-[4.1vw] max-lg:hidden">
						<BracketLink label="BOUNTIES" href="/bounties" />
						<BracketLink label="DASHBOARD" href="/dashboard" />
					</div>
					<div className="flex items-center gap-[4.1vw] relative">
						<div onClick={handleConnect}>
							<BracketLink label={contentText} />
						</div>

						{/* Disconnect Bubble */}
						<motion.div
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
							onClick={handleDisconnect}
						>
							<span className="label-14-mono">DISCONNECT</span>
						</motion.div>

						<div 
							onClick={() => setMenuOpen(!menuOpen)}
							className="relative flex h-[14px] w-[35px] flex-col justify-between overflow-hidden cursor-pointer max-sm:h-[13px] max-sm:w-[32px] group z-[100]"
						>
							<div className={`bg-white h-[1.5px] w-full max-sm:h-[1px] transition-all duration-300 ${menuOpen ? "rotate-45 translate-y-[6.25px]" : ""}`} />
							<div className={`bg-white h-[1.5px] w-full max-sm:h-[1px] transition-all duration-300 ${menuOpen ? "opacity-0" : ""}`} />
							<div className={`bg-white h-[1.5px] w-full max-sm:h-[1px] transition-all duration-300 ${menuOpen ? "-rotate-45 -translate-y-[6.25px]" : ""}`} />
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
			className="bracket-link relative uppercase tracking-widest text-xs md:text-sm font-bold flex gap-[0.5em]"
			initial="rest"
			whileHover="hover"
			animate="rest"
		>
			<motion.span
				variants={{
					rest: { opacity: 0.3, x: 0 },
					hover: { opacity: 1, x: -4, color: "#f13242" },
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
					hover: { opacity: 1, x: 4, color: "#f13242" },
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
