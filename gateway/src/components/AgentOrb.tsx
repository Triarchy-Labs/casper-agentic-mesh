"use client";

import React, { useEffect, useState, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";

export type AgentState = "idle" | "thinking" | "working" | "success" | "error" | "sneaky" | "sad" | "exhausted" | "surrender" | "danger" | "typing";

interface AgentOrbProps {
	state: AgentState;
	size?: number;
}

export function AgentOrb({ state, size = 120 }: AgentOrbProps) {
	const [blink, setBlink] = useState(false);
	const [mousePos, setMousePos] = useState({ x: 0, y: 0 });
	const orbRef = useRef<HTMLDivElement>(null);

	// Blinking logic
	useEffect(() => {
		const blinkInterval = setInterval(() => {
			if (state === "idle" || state === "thinking" || state === "sneaky" || state === "sad" || state === "exhausted" || state === "typing") {
				setBlink(true);
				setTimeout(() => setBlink(false), state === "exhausted" ? 400 : 150);
			}
		}, state === "exhausted" ? Math.random() * 8000 + 4000 : Math.random() * 4000 + 2000); 

		return () => clearInterval(blinkInterval);
	}, [state]);

	// Mouse tracking logic
	useEffect(() => {
		const handleMouseMove = (e: MouseEvent) => {
			if (state !== "idle" && state !== "sneaky" && state !== "sad") {
				setMousePos({ x: 0, y: 0 });
				return;
			}
			if (!orbRef.current) return;
			
			const rect = orbRef.current.getBoundingClientRect();
			const centerX = rect.left + rect.width / 2;
			const centerY = rect.top + rect.height / 2;
			
			const maxOffset = 20; 
			const deltaX = e.clientX - centerX;
			const deltaY = e.clientY - centerY;
			const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);
			
			const maxDist = 400; 
			const ratio = Math.min(distance / maxDist, 1);
			const angle = Math.atan2(deltaY, deltaX);
			
			setMousePos({
				x: Math.cos(angle) * ratio * maxOffset,
				y: Math.sin(angle) * ratio * maxOffset
			});
		};

		window.addEventListener("mousemove", handleMouseMove);
		return () => window.removeEventListener("mousemove", handleMouseMove);
	}, [state]);

	// Container physics
	const containerVariants = {
		idle:      { x: 0, y: 0, scale: 1, rotate: 0, transition: { duration: 0.5, ease: "easeOut" as const } },
		thinking:  { x: [0, -15, -15, 15, 15, 0], y: [0, -8, -8, -8, -8, 0], transition: { duration: 4, repeat: Infinity, ease: "easeInOut" as const } },
		working:   { x: 0, y: [-1, 1, -1], transition: { duration: 1.8, repeat: Infinity, ease: "easeInOut" as const } },
		typing:    { x: 0, y: 0, scale: 0.8, transition: { duration: 0.3, ease: "easeOut" as const } },
		sneaky:    { x: 18, y: 4, scale: 0.95, transition: { duration: 0.6, ease: "easeOut" as const } },
		success:   { x: 0, y: [0, -15, 0], transition: { duration: 1.2, repeat: Infinity, repeatDelay: 1, ease: "easeInOut" as const } },
		error:     { x: [-6, 6, -6, 6, 0], y: 0, transition: { duration: 0.8, repeat: Infinity, ease: "easeInOut" as const } },
		sad:       { x: 0, y: 8, transition: { duration: 1.5, ease: "easeOut" as const } },
		exhausted: { x: 0, y: 12, scale: 0.95, transition: { duration: 2.5, ease: "easeOut" as const } },
		surrender: { x: 0, y: 0, transition: { duration: 0.8, ease: "easeOut" as const } },
		danger:    { x: [-4, 4, -4, 4, 0], y: 0, transition: { duration: 0.6, repeat: Infinity, ease: "easeInOut" as const } }
	};

	// Pure White Eye shapes without black UI masks
    // IMPORTANT: ALL borderRadius values MUST have 4 explicit values so framer-motion interpolates without snapping!
	const leftEyeVariants = {
		idle: { height: blink ? 2 : 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		typing: { height: blink ? 2 : 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" }, 
		thinking: { height: blink ? 2 : 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		working: { height: 9, width: 24, rotate: 0, borderRadius: "4px", backgroundColor: "#00ff41" }, 
		sneaky: { height: 27, width: 18, rotate: -5, borderRadius: "12px 12px 12px 12px", backgroundColor: "#fff" },
		success: { height: blink ? 2 : 33, width: 24, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#00ff41" },
		error: { height: 15, width: 27, rotate: 30, borderRadius: "6px 6px 15px 15px", backgroundColor: "#fff" }, 
		sad: { height: 27, width: 24, rotate: -15, borderRadius: "6px 6px 18px 3px", backgroundColor: "#fff" }, 
		exhausted: { height: blink ? 2 : 12, width: 21, rotate: 0, borderRadius: "6px 6px 15px 15px", backgroundColor: "#fff" },
		surrender: { height: 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		danger: { height: 12, width: 27, rotate: 30, borderRadius: "3px 3px 3px 3px", backgroundColor: "#ff003c" } 
	};
    
	const rightEyeVariants = {
		idle: { height: blink ? 2 : 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		typing: { height: blink ? 2 : 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		thinking: { height: blink ? 2 : 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		working: { height: 9, width: 24, rotate: 0, borderRadius: "4px", backgroundColor: "#00ff41" }, 
		sneaky: { height: 36, width: 21, rotate: 5, borderRadius: "12px 12px 12px 12px", backgroundColor: "#fff" }, 
		success: { height: blink ? 2 : 33, width: 24, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#00ff41" },
		error: { height: 15, width: 27, rotate: -30, borderRadius: "6px 6px 15px 15px", backgroundColor: "#fff" }, 
		sad: { height: 27, width: 24, rotate: 15, borderRadius: "6px 6px 3px 18px", backgroundColor: "#fff" }, 
		exhausted: { height: blink ? 2 : 12, width: 21, rotate: 0, borderRadius: "6px 6px 15px 15px", backgroundColor: "#fff" },
		surrender: { height: 39, width: 21, rotate: 0, borderRadius: "15px 15px 15px 15px", backgroundColor: "#fff" },
		danger: { height: 12, width: 27, rotate: -30, borderRadius: "3px 3px 3px 3px", backgroundColor: "#ff003c" } 
	};

	return (
		<div style={{ width: size, height: size, display: "flex", alignItems: "center", justifyContent: "center" }}>
			<div style={{ width: 120, height: 120, transform: `scale(${size / 120})`, transformOrigin: "center" }} className="relative flex items-center justify-center perspective-1000">
			
			{/* Flag Props mapped to State */}
			<AnimatePresence>
				{state === "surrender" && (
					<motion.div 
						initial={{ rotate: -90, y: 30, opacity: 0 }}
						animate={{ rotate: 0, y: 0, opacity: 1 }}
						exit={{ opacity: 0, transition: { duration: 0.2 } }}
						className="absolute -right-12 top-4 origin-bottom-left z-20 pointer-events-none"
					>
						<div className="w-1.5 h-16 bg-[#555] absolute left-0 top-0 rounded" />
						<motion.div 
							animate={{ skewY: [-5, 5, -5] }}
							transition={{ repeat: Infinity, duration: 0.6, ease: "easeInOut" }}
							className="w-12 h-8 bg-white absolute left-1.5 top-0 rounded-r-md shadow-lg"
						/>
					</motion.div>
				)}
				{state === "danger" && (
					<motion.div 
						initial={{ rotate: 90, y: 30, opacity: 0 }}
						animate={{ rotate: 0, y: 0, opacity: 1 }}
						exit={{ opacity: 0, transition: { duration: 0.2 } }}
						className="absolute -left-16 top-4 origin-bottom-right z-20 pointer-events-none"
					>
						<div className="w-1.5 h-16 bg-[#555] absolute right-0 top-0 rounded" />
						<motion.div 
							animate={{ skewY: [5, -5, 5] }}
							transition={{ repeat: Infinity, duration: 0.4, ease: "easeInOut" }}
							className="w-14 h-10 bg-black border border-white/20 absolute right-1.5 top-0 rounded-l-md shadow-[0_0_20px_#ff003c] flex items-center justify-center text-[10px]"
						>
							☠️
						</motion.div>
					</motion.div>
				)}

                {/* Freighter Keys Prop: Appears when 'success', orbiting the orb */}
                {state === "success" && (
                    <motion.div 
						initial={{ scale: 0, opacity: 0 }}
						animate={{ scale: 1, opacity: 1, rotate: 360 }}
						exit={{ opacity: 0, transition: { duration: 0.2 } }}
						className="absolute w-[240px] h-[240px] z-30 pointer-events-none"
                        transition={{ rotate: { duration: 4, repeat: Infinity, ease: "linear" } }}
					>
                        <motion.div 
                            className="absolute -right-2 top-1/2 origin-center"
                            initial={{ rotate: 0 }}
                            animate={{ rotate: -360, y: ["-50%", "calc(-50% - 6px)", "-50%"] }}
                            transition={{ rotate: { duration: 4, repeat: Infinity, ease: "linear" }, y: { duration: 1.5, repeat: Infinity, ease: "easeInOut" } }}
                        >
                            <svg 
                                width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="#7e4ccb" strokeWidth="2" strokeLinecap="round" className="drop-shadow-[0_0_12px_#7e4ccb] -rotate-45"
                            >
                                <path d="M21 2l-2 2v4l2 2" />
                                <path d="M14 6h3" />
                                <path d="M14 10h5" />
                                <circle cx="7" cy="15" r="5" />
                                <path d="M10.5 11.5L14 8" />
                            </svg>
                        </motion.div>
                    </motion.div>
                )}
			</AnimatePresence>

			<motion.div 
				ref={orbRef}
				style={{ width: 120, height: 120 }} 
				className="relative z-10 rounded-full shadow-[inset_0_-10px_20px_rgba(0,0,0,0.8),_0_0_15px_rgba(0,0,0,0.5)] border border-white/5 overflow-hidden flex items-center justify-center bg-[#0a0a0a]"
                animate={{ 
                    boxShadow: state === "success" 
                        ? "inset 0 -10px 20px rgba(0,0,0,0.8), 0 0 30px rgba(0, 255, 65, 0.4), 0 0 0 2px rgba(0, 255, 65, 0.5)" 
                        : "inset 0 -10px 20px rgba(0,0,0,0.8), 0 0 15px rgba(0,0,0,0.5), 0 0 0 1px rgba(255,255,255,0.05)"
                }}
			>
				<div className="absolute top-[10%] left-[20%] w-[40%] h-[20%] bg-white/10 rounded-full blur-[2px] rotate-[-20deg] pointer-events-none z-20" />

				<motion.div
					className="absolute inset-0 flex items-center justify-center z-10"
					animate={{ x: mousePos.x, y: mousePos.y }}
					transition={{ type: "spring", stiffness: 100, damping: 25 }}
				>
					<motion.div className="flex gap-8 z-10" variants={containerVariants} animate={state} style={{ transformStyle: 'preserve-3d' }}>
						{/* Left Eye */}
						<motion.div variants={leftEyeVariants} animate={state} transition={{ type: "spring", stiffness: 300, damping: 20 }} />
						
						{/* Right Eye */}
						<motion.div variants={rightEyeVariants} animate={state} transition={{ type: "spring", stiffness: 300, damping: 20 }} />
					</motion.div>
				</motion.div>
				
				<AnimatePresence>
					{state === "working" && (
						<motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 bg-[#00ff41]/5 mix-blend-overlay animate-pulse pointer-events-none z-30" />
					)}
					{state === "error" && (
						<motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 shadow-[inset_0_0_20px_#ff003c] mix-blend-overlay pointer-events-none z-30" />
					)}
					{state === "danger" && (
						<motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 shadow-[inset_0_0_30px_#ff003c] mix-blend-overlay pointer-events-none z-30 animate-pulse" />
					)}
					{state === "sad" && (
						<motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 bg-[#00d4ff]/10 mix-blend-overlay pointer-events-none z-30" />
					)}
				</AnimatePresence>
			</motion.div>
		</div>
		</div>
	);
}
