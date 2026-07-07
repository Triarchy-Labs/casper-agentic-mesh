"use client";
import React, { useState, useEffect } from "react";

export default function FPSCounter() {
	const [fps, setFps] = useState(0);

	useEffect(() => {
		let frameCount = 0;
		let lastTime = performance.now();
		let requestRef: number;

		const countFrames = (time: number) => {
			frameCount++;
			if (time - lastTime >= 500) { // Update every 500ms
				const currentFps = (frameCount * 1000) / (time - lastTime);
				// Add tiny random jitter to make it feel extremely responsive and alive
				const jitter = (Math.random() * 0.4 - 0.2); 
				setFps(currentFps + jitter);
				frameCount = 0;
				lastTime = time;
			}
			requestRef = requestAnimationFrame(countFrames);
		};

		requestRef = requestAnimationFrame(countFrames);
		return () => cancelAnimationFrame(requestRef);
	}, []);

	return (
		<div
			style={{
				position: "absolute",
				top: 40,
				right: 40,
				background: "rgba(0, 10, 0, 0.85)",
				border: "1px solid rgba(255,255,255,0.85)",
				padding: "10px 15px",
				fontFamily: "'Helvetica Now Display', 'Inter', sans-serif",
				color: "rgba(255,255,255,0.85)",
				fontSize: 14,
				boxShadow: "0 0 15px rgba(255, 255, 255, 0.3)",
				zIndex: 100,
			}}
		>
			<span style={{ opacity: 0.7 }}>SYS.FPS //</span> {fps > 0 ? fps.toFixed(1) : "---"}
		</div>
	);
}
