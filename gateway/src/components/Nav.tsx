"use client";

import React, { useState } from "react";
import { motion } from "framer-motion";
import {
  isConnected as checkFreighterConnected,
  requestAccess,
} from "@stellar/freighter-api";
import { AgentOrb } from "@/components/AgentOrb";
import { useEffect } from "react";

export function Nav() {
	const [connected, setConnected] = useState(false);
	const [pubKey, setPubKey] = useState("");
	const [connecting, setConnecting] = useState(false);
	const [freighterMissing, setFreighterMissing] = useState(false);
    const [hoverLogo, setHoverLogo] = useState(false);
    const [showDisconnect, setShowDisconnect] = useState(false);

    // Auto-connect on load across Dashboard and Page
    useEffect(() => {
		const checkConn = async () => {
			try {
				const status = await checkFreighterConnected();
				if (status.isConnected) {
					const access = await requestAccess();
					if (access.address) {
						const key = access.address;
						setPubKey(key.substring(0, 4) + "..." + key.substring(key.length - 4));
						setConnected(true);
					}
				}
			} catch (e) {}
		};
		checkConn();
	}, []);

	const handleConnect = async () => {
        if (connected) {
            setShowDisconnect(!showDisconnect);
            return;
        }

		if (freighterMissing) {
			window.open("https://chromewebstore.google.com/detail/freighter/bcacfldlkkdogcmkkibnjlakofdplcbk", "_blank");
			return;
		}

		setConnecting(true);
		try {
            const status = await checkFreighterConnected();
            if (status.isConnected) {
                const access = await requestAccess();
                if (access.error) {
                    console.log("Freighter connection rejected.", access.error);
                } else if (access.address) {
                    const key = access.address;
                    setPubKey(key.substring(0, 4) + "..." + key.substring(key.length - 4));
                    setConnected(true);
                }
            } else {
                setFreighterMissing(true);
            }
		} catch (error) {
			console.error("Freighter connect failed", error);
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
				padding: "0 var(--space-nav-x)",
				zIndex: 100,
				background: "linear-gradient(to bottom, rgba(0,0,0,0.4) 0%, rgba(0,0,0,0) 100%)",
				pointerEvents: "none",
			}}
		>
			<div 
                style={{ pointerEvents: "auto", display: "flex", alignItems: "center", gap: "12px", cursor: "pointer", position: "relative" }}
                onClick={() => window.location.href = "/"}
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
                    <span className="hidden sm:inline" style={{ 
                        fontFamily: '"Space Mono", monospace', 
                        fontSize: "0.85rem", 
                        fontWeight: "bold",
                        letterSpacing: "0.1em",
                        textTransform: "uppercase",
                        color: "#fff",
                        paddingLeft: "4px"
                    }}>
                        [ RETURN TO HQ ]
                    </span>
                </motion.div>
			</div>

			<div className="nav-actions" style={{ pointerEvents: "auto", display: "flex", gap: "24px", alignItems: "center", position: "relative" }}>
				<button
					onClick={() => window.location.href = "/dashboard"}
					style={{
						background: "rgba(10,10,10,0.7)",
						color: "rgba(255,255,255,0.7)",
						border: "1px solid rgba(255,255,255,0.2)",
						padding: "8px 24px",
						borderRadius: "40px",
						cursor: "pointer",
						fontFamily: '"Space Mono", monospace',
						fontSize: "0.8rem",
						fontWeight: "bold",
						letterSpacing: "0.1em",
						transition: "all 0.2s ease",
						backdropFilter: "blur(12px)",
					}}
					onMouseEnter={(e) => { 
						e.currentTarget.style.backgroundColor = "rgba(0, 255, 65, 0.15)";
						e.currentTarget.style.color = "#00ff41";
						e.currentTarget.style.borderColor = "rgba(0, 255, 65, 0.4)";
					}}
					onMouseLeave={(e) => { 
						e.currentTarget.style.backgroundColor = "rgba(10,10,10,0.7)";
						e.currentTarget.style.color = "rgba(255,255,255,0.7)";
						e.currentTarget.style.borderColor = "rgba(255,255,255,0.2)";
					}}
				>
					[ DASHBOARD ]
				</button>
                <div style={{ position: "relative" }}>
                    <button
                        onClick={handleConnect}
                        className={`connect-btn-base ${connected || freighterMissing ? "connect-state-true" : "connect-state-false connect-btn-hover-fx"}`}
                        style={{
                            transform: "scale(1)",
                            ...(freighterMissing ? { backgroundColor: "rgba(255,255,255,0.05)", borderColor: "rgba(255,255,255,0.1)", color: "#aaa" } : {})
                        }}
                        onMouseDown={(e) => e.currentTarget.style.transform = "scale(0.95)"}
                        onMouseUp={(e) => e.currentTarget.style.transform = "scale(1.05)"}
                        onMouseEnter={(e) => e.currentTarget.style.transform = "scale(1.05)"}
                        onMouseLeave={(e) => e.currentTarget.style.transform = "scale(1)"}
                    >
                        <span>{connecting ? "[ CONNECTING... ]" : connected ? `[ ${pubKey} ]` : freighterMissing ? "[ GET FREIGHTER ]" : "[ CONNECT WALLET ]"}</span>
                    </button>

                    {/* Disconnect Bubble */}
                    <motion.div
                        initial={{ opacity: 0, y: -10, pointerEvents: "none" }}
                        animate={{ opacity: showDisconnect ? 1 : 0, y: showDisconnect ? 10 : -10, pointerEvents: showDisconnect ? "auto" : "none" }}
                        style={{
                            position: "absolute",
                            top: "100%",
                            right: 0,
                            background: "rgba(255, 0, 60, 0.1)",
                            border: "1px solid rgba(255, 0, 60, 0.5)",
                            backdropFilter: "blur(12px)",
                            padding: "8px 16px",
                            borderRadius: "12px",
                            cursor: "pointer",
                            color: "#ff003c"
                        }}
                        onClick={handleDisconnect}
                    >
                        <span style={{ fontFamily: '"Space Mono", monospace', fontSize: "0.75rem", fontWeight: "bold" }}>DISCONNECT</span>
                    </motion.div>
                </div>
			</div>
		</motion.div>
	);
}
