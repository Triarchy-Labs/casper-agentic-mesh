"use client";

import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { AgentOrb } from "@/components/AgentOrb";
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

	return (
		<motion.div
			initial={{ opacity: 0, y: -20 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ duration: 1.5, ease: [0.16, 1, 0.3, 1] }}
			style={{
				position: "fixed",
				top: 0,
				left: 0,
				width: "100%",
				height: "80px",
				display: "flex",
				justifyContent: "space-between",
				alignItems: "center",
				padding: "0 24px",
				zIndex: 100,
				background: "linear-gradient(to bottom, rgba(0,0,0,0.8) 0%, rgba(0,0,0,0) 100%)",
				pointerEvents: "none",
			}}
		>
			<Link 
                href="/"
                style={{ pointerEvents: "auto", display: "flex", alignItems: "center", gap: "12px", cursor: "pointer", position: "relative", textDecoration: "none" }}
                onMouseEnter={() => setHoverLogo(true)}
                onMouseLeave={() => setHoverLogo(false)}
            >
				<motion.div 
                    animate={{ scale: hoverLogo ? 1.1 : 1 }}
                    transition={{ type: "spring", stiffness: 300, damping: 20 }}
                    style={{ flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', width: "32px", height: "32px" }} 
                >
                    <AgentOrb state={hoverLogo ? "typing" : "idle"} size={28} />
                </motion.div>
				<motion.div
                    initial={{ width: 0, opacity: 0 }}
                    animate={{ width: hoverLogo ? "auto" : 0, opacity: hoverLogo ? 1 : 0 }}
                    style={{ overflow: "hidden", whiteSpace: "nowrap" }}
                    transition={{ type: "spring", stiffness: 200, damping: 20 }}
                >
                    <span className="hidden sm:inline label-14-mono text-[var(--gray-1000)] pl-[8px]">
                        [ RETURN TO HQ ]
                    </span>
                </motion.div>
			</Link>

			<div style={{ pointerEvents: "auto", display: "flex", gap: "24px", alignItems: "center", position: "relative" }}>
				<Link
					href="/bounties"
					className="button-secondary label-14-mono backdrop-blur-md"
				>
					[ BOUNTIES ]
				</Link>
				<Link
					href="/dashboard"
					className="button-secondary label-14-mono backdrop-blur-md"
				>
					[ DASHBOARD ]
				</Link>
                <div style={{ position: "relative" }}>
                    <button
                        onClick={handleConnect}
                        className="button-primary label-14-mono transition-transform active:scale-95"
                    >
                        <span>{connecting ? "[ CONNECTING... ]" : connected ? `[ ${pubKey} ]` : walletMissing ? "[ GET CASPER WALLET ]" : "[ CONNECT WALLET ]"}</span>
                    </button>

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
                </div>
			</div>
		</motion.div>
	);
}
