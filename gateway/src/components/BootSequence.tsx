"use client";
import React, { useState, useEffect } from "react";
import { motion } from "framer-motion";

export default function BootSequence({ onComplete }: { onComplete: () => void }) {
	const [progress, setProgress] = useState(0);

	useEffect(() => {
		let current = 0;
		const interval = setInterval(() => {
			current += Math.floor(Math.random() * 15) + 3; // Fast nonlinear growth
			if (current >= 100) {
				current = 100;
				clearInterval(interval);
				setTimeout(() => onComplete(), 400); // Tiny hold at 100% then trigger dissolve
			}
			setProgress(current);
		}, 70);
		return () => clearInterval(interval);
	}, [onComplete]);

	return (
		<motion.div
            // Epic Lusion-style dissolve
			exit={{ opacity: 0, scale: 1.1, filter: "blur(20px)" }}
			transition={{ duration: 0.8, ease: "easeIn" }}
			style={{
				position: "fixed",
				inset: 0,
				background: "#010201",
				zIndex: 99999,
				display: "flex",
				flexDirection: "column",
				alignItems: "center",
				justifyContent: "center",
				color: "rgba(255,255,255,0.85)",
				fontFamily: "monospace",
			}}
		>
			<motion.div
				initial={{ opacity: 0, scale: 0.8 }}
				animate={{ opacity: 1, scale: 1 }}
				transition={{ duration: 0.5 }}
				style={{ 
                    fontSize: "8vw", 
                    fontWeight: "bold", 
                    textShadow: "0 0 40px rgba(255,255,255,0.85)",
                    letterSpacing: "-0.05em"
                }}
			>
				{progress}%
			</motion.div>
			
            <motion.div
                initial={{ opacity: 0 }}
				animate={{ opacity: 1 }}
				style={{
					marginTop: "2rem",
					width: "300px",
					height: "2px",
					background: "rgba(255, 255, 255, 0.2)",
					position: "relative",
					overflow: "hidden"
				}}
			>
				<div style={{
					position: "absolute",
					top: 0, left: 0, bottom: 0,
					width: `${progress}%`,
					background: "rgba(255,255,255,0.85)",
					boxShadow: "0 0 15px rgba(255,255,255,0.85)",
                    transition: "width 0.1s linear"
				}} />
			</motion.div>
            <motion.div 
                initial={{ opacity: 0 }}
				animate={{ opacity: 1 }}
                style={{ 
                    marginTop: "1.5rem", 
                    letterSpacing: "0.2em", 
                    fontSize: "0.8rem", 
                    color: "rgba(255, 255, 255, 0.6)" 
                }}
            >
				INITIALIZING L402 MESH PROTOCOL...
			</motion.div>
		</motion.div>
	);
}
