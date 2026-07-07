"use client";

import React, { useEffect, useState, useRef } from "react";
import { motion } from "framer-motion";

export function SystemWidget() {
	const [fps, setFps] = useState(60);
	const [timeStr, setTimeStr] = useState("00:00:00:000");
	
	const fpsRef = useRef({ count: 0, lastTime: 0 });

	useEffect(() => {
		fpsRef.current.lastTime = performance.now();
		let frameId: number;

		const loop = (now: number) => {
			// Update Time
			const d = new Date();
			const time = d.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit' });
			setTimeStr(time);

			// Update FPS
			fpsRef.current.count++;
			if (now - fpsRef.current.lastTime >= 1000) {
				setFps(Math.round((fpsRef.current.count * 1000) / (now - fpsRef.current.lastTime)));
				fpsRef.current.count = 0;
				fpsRef.current.lastTime = now;
			}
			
			frameId = requestAnimationFrame(loop);
		};

		frameId = requestAnimationFrame(loop);
		return () => cancelAnimationFrame(frameId);
	}, []);

	return (
		<motion.div
			initial={{ opacity: 0, x: -20 }}
			animate={{ opacity: 1, x: 0 }}
			transition={{ duration: 1.5, ease: [0.16, 1, 0.3, 1] }}
			style={{
				position: "fixed",
				bottom: "25vh",
				left: "12px",
				display: "flex",
				alignItems: "center",
				fontSize: "0.6rem",
				fontFamily: '"Space Mono", monospace',
				color: "rgba(255,255,255,0.6)",
				textTransform: "uppercase",
				letterSpacing: "0.1em",
				zIndex: 50,
				pointerEvents: "none",
				mixBlendMode: "screen",
                writingMode: "vertical-rl",
                transform: "rotate(180deg)",
			}}
		>
			<span style={{ color: "#fff", display: "inline-block" }}>[ {timeStr}</span>
			<span style={{ margin: "10px 0", color: "rgba(255,255,255,0.2)", display: "inline-block" }}>{"//"}</span>
			<span style={{ color: "rgba(255,255,255,0.6)", display: "inline-block" }}>FPS</span>
			<span style={{ 
				marginTop: "10px",
				color: fps >= 55 ? "#00ff41" : fps >= 30 ? "#ffb000" : "#ff003c",
				fontWeight: "bold",
				textShadow: fps >= 55 ? "0 0 10px rgba(0,255,65,0.3)" : "none",
				display: "inline-block"
			}}>
				{fps.toString().padStart(2, "0")}
			</span>
			<span style={{ marginTop: "6px", color: "#fff", display: "inline-block" }}>]</span>
		</motion.div>
	);
}
