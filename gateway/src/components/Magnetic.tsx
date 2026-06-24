"use client";
import { motion } from "framer-motion";
import React, { useRef, useState } from "react";

export default function Magnetic({ children, strength = 0.3, style, className }: { children: React.ReactNode; strength?: number; style?: React.CSSProperties; className?: string }) {
	const ref = useRef<HTMLDivElement>(null);
	const [position, setPosition] = useState({ x: 0, y: 0 });

	const handleMouse = (e: React.MouseEvent<HTMLDivElement>) => {
		if (!ref.current) return;
		const { clientX, clientY } = e;
		const { height, width, left, top } = ref.current.getBoundingClientRect();
		const middleX = clientX - (left + width / 2);
		const middleY = clientY - (top + height / 2);
		setPosition({ x: middleX * strength, y: middleY * strength });
	};

	const reset = () => {
		setPosition({ x: 0, y: 0 });
	};

	return (
		<motion.div
			ref={ref}
			onMouseMove={handleMouse}
			onMouseLeave={reset}
			animate={{ x: position.x, y: position.y }}
			transition={{ type: "spring", stiffness: 150, damping: 15, mass: 0.1 }}
			style={{ display: "inline-block", position: "relative", ...style }}
			className={className}
		>
			<div style={{ position: "absolute", top: -40, left: -40, right: -40, bottom: -40, zIndex: 1 }} />
			<div style={{ position: "relative", zIndex: 2 }}>
				{children}
			</div>
		</motion.div>
	);
}
