"use client";
import React, { useEffect, useRef, useState } from "react";

export default function CustomCursor() {
	const dotRef = useRef<HTMLDivElement>(null);
	const sq1Ref = useRef<HTMLDivElement>(null);
	const sq2Ref = useRef<HTMLDivElement>(null);
	const [isHovering, setIsHovering] = useState(false);
	const [isVisible, setIsVisible] = useState(false);
	const [isTouch, setIsTouch] = useState(false);
	const mousePos = useRef({ x: -100, y: -100 });
	const currentPos = useRef({ x: -100, y: -100, scale: 1 });
	const rafId = useRef<number>(0);
	const isHoveringRef = useRef(false);

	// Floating squares — independent position refs for parallax drift
	const sq1Pos = useRef({ x: -100, y: -100 });
	const sq2Pos = useRef({ x: -100, y: -100 });
	const isOverCardRef = useRef(false);
	const [isOverCard, setIsOverCard] = useState(false);
	// Smooth opacity/scale for card-hover squares
	const sq1Vis = useRef(0);
	const sq2Vis = useRef(0);

	// Lerp smoothing factors
	const LERP = 0.15;
	const SQ1_LERP = 0.08;
	const SQ2_LERP = 0.12;
	const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

	// Square offsets from cursor
	const SQ1_OFFSET = { x: 12, y: 8 };
	const SQ2_OFFSET = { x: -10, y: 14 };

	// Check if this is a touch device on mount
	useEffect(() => {
		const isT = window.matchMedia("(pointer: coarse)").matches;
		setTimeout(() => setIsTouch(isT), 0);
	}, []);

	// Keep hover ref in sync
	useEffect(() => {
		isHoveringRef.current = isHovering;
	}, [isHovering]);

	// Keep card-hover ref in sync
	useEffect(() => {
		isOverCardRef.current = isOverCard;
	}, [isOverCard]);

	useEffect(() => {
		if (isTouch) return;

		// Hide native cursor
		const style = document.createElement("style");
		style.id = "hide-native-cursor";
		style.textContent = "*, *::before, *::after { cursor: none !important; }";
		document.head.appendChild(style);

		const updateMouse = (e: MouseEvent) => {
			if (!isVisible) setIsVisible(true);
			mousePos.current = { x: e.clientX, y: e.clientY };
		};

		const handleMouseOver = (e: MouseEvent) => {
			const target = e.target as HTMLElement;
			if (!target) return;
			const cs = window.getComputedStyle(target);
			setIsHovering(
				cs.cursor === "pointer" ||
				cs.cursor === "crosshair" ||
				!!target.closest("button") ||
				!!target.closest("a")
			);
			// Track editorial-panel hover for floating squares
			setIsOverCard(!!target.closest(".editorial-panel"));
		};

		// RAF loop — calculates physics
		const tick = () => {
			currentPos.current.x = lerp(currentPos.current.x, mousePos.current.x, LERP);
			currentPos.current.y = lerp(currentPos.current.y, mousePos.current.y, LERP);

			// Smoothly animate the scale instead of instant snapping
			const targetScale = isHoveringRef.current ? 1.3 : 1;
			currentPos.current.scale = lerp(currentPos.current.scale, targetScale, LERP * 0.4);

			if (dotRef.current) {
				// Offset is 10px because base width/height is 20
				dotRef.current.style.transform = `translate(${currentPos.current.x - 10}px, ${currentPos.current.y - 10}px) scale(${currentPos.current.scale})`;
			}

			// Floating squares — drift toward cursor+offset with slower lerp
			const targetVis = isOverCardRef.current ? 1 : 0;
			sq1Vis.current = lerp(sq1Vis.current, targetVis, 0.08);
			sq2Vis.current = lerp(sq2Vis.current, targetVis, 0.06);

			sq1Pos.current.x = lerp(sq1Pos.current.x, mousePos.current.x + SQ1_OFFSET.x, SQ1_LERP);
			sq1Pos.current.y = lerp(sq1Pos.current.y, mousePos.current.y + SQ1_OFFSET.y, SQ1_LERP);
			sq2Pos.current.x = lerp(sq2Pos.current.x, mousePos.current.x + SQ2_OFFSET.x, SQ2_LERP);
			sq2Pos.current.y = lerp(sq2Pos.current.y, mousePos.current.y + SQ2_OFFSET.y, SQ2_LERP);

			if (sq1Ref.current) {
				const o = sq1Vis.current;
				sq1Ref.current.style.transform = `translate(${sq1Pos.current.x}px, ${sq1Pos.current.y}px) scale(${o})`;
				sq1Ref.current.style.opacity = String(o * 0.15);
			}
			if (sq2Ref.current) {
				const o = sq2Vis.current;
				sq2Ref.current.style.transform = `translate(${sq2Pos.current.x}px, ${sq2Pos.current.y}px) scale(${o})`;
				sq2Ref.current.style.opacity = String(o * 0.25);
			}

			rafId.current = requestAnimationFrame(tick);
		};

		window.addEventListener("mousemove", updateMouse, { passive: true });
		window.addEventListener("mouseover", handleMouseOver, { passive: true });
		rafId.current = requestAnimationFrame(tick);

		return () => {
			window.removeEventListener("mousemove", updateMouse);
			window.removeEventListener("mouseover", handleMouseOver);
			cancelAnimationFrame(rafId.current);
			const el = document.getElementById("hide-native-cursor");
			if (el) el.remove();
		};
	}, [isVisible, isTouch]);

	if (isTouch || !isVisible) return null;

	return (
		<>
			{/* Main cursor dot */}
			<div
				ref={dotRef}
				style={{
					position: "fixed",
					top: 0,
					left: 0,
					width: 20,
					height: 20,
					borderRadius: "0px",
					border: "1.5px solid var(--red-700)",
					backgroundColor: isHovering ? "rgba(241, 50, 66, 0.08)" : "transparent",
					pointerEvents: "none",
					zIndex: 99999,
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					boxShadow: isHovering ? "0 0 10px rgba(241, 50, 66, 0.4)" : "none",
					transition: "background-color 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease",
					willChange: "transform",
				}}
			>
				<div
					style={{
						width: 4,
						height: 4,
						backgroundColor: "var(--red-700)",
						transition: "transform 0.3s ease",
						transform: isHovering ? "scale(1.5)" : "scale(1)"
					}}
				/>
			</div>

			{/* Floating square 1 — filled white, 8x8 */}
			<div
				ref={sq1Ref}
				style={{
					position: "fixed",
					top: 0,
					left: 0,
					width: 8,
					height: 8,
					backgroundColor: "#fff",
					opacity: 0,
					pointerEvents: "none",
					zIndex: 9998,
					willChange: "transform, opacity",
				}}
			/>

			{/* Floating square 2 — border-only, 6x6 */}
			<div
				ref={sq2Ref}
				style={{
					position: "fixed",
					top: 0,
					left: 0,
					width: 6,
					height: 6,
					backgroundColor: "transparent",
					border: "1px solid rgba(255, 255, 255, 0.25)",
					opacity: 0,
					pointerEvents: "none",
					zIndex: 9998,
					willChange: "transform, opacity",
				}}
			/>
		</>
	);
}
