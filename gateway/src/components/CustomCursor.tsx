"use client";
import React, { useEffect, useRef, useState } from "react";

export default function CustomCursor() {
	const dotRef = useRef<HTMLDivElement>(null);
	const arrowRef = useRef<HTMLDivElement>(null);
	const labelRef = useRef<HTMLDivElement>(null);
	const [isHovering, setIsHovering] = useState(false);
	const [isVisible, setIsVisible] = useState(false);
	const [isTouch, setIsTouch] = useState(false);
	const mousePos = useRef({ x: -100, y: -100 });
	const currentPos = useRef({ x: -100, y: -100, scale: 1 });
	// Slower-lerp positions for the drifting followers
	const arrowPos = useRef({ x: -100, y: -100 });
	const labelPos = useRef({ x: -100, y: -100 });
	const rafId = useRef<number>(0);
	const isHoveringRef = useRef(false);
	const isOverCardRef = useRef(false);
	const [isOverCard, setIsOverCard] = useState(false);
	const cardOpacity = useRef(0);

	const LERP = 0.15;
	const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

	useEffect(() => {
		const isT = window.matchMedia("(pointer: coarse)").matches;
		setTimeout(() => setIsTouch(isT), 0);
	}, []);

	useEffect(() => {
		isHoveringRef.current = isHovering;
	}, [isHovering]);

	useEffect(() => {
		isOverCardRef.current = isOverCard;
	}, [isOverCard]);

	useEffect(() => {
		if (isTouch) return;

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
			const hovering =
				cs.cursor === "pointer" ||
				cs.cursor === "crosshair" ||
				!!target.closest("button") ||
				!!target.closest("a");
			setIsHovering(hovering);
			// Check if over an editorial-panel card or interactive button
			const overCard = !!target.closest(".editorial-panel") || !!target.closest(".button-primary") || !!target.closest(".button-secondary") || !!target.closest(".btn-ghost");
			setIsOverCard(overCard);
		};

		const tick = () => {
			// Main cursor dot — fast lerp
			currentPos.current.x = lerp(currentPos.current.x, mousePos.current.x, LERP);
			currentPos.current.y = lerp(currentPos.current.y, mousePos.current.y, LERP);
			const targetScale = isHoveringRef.current ? 1.3 : 1;
			currentPos.current.scale = lerp(currentPos.current.scale, targetScale, LERP * 0.4);

			if (dotRef.current) {
				dotRef.current.style.transform = `translate(${currentPos.current.x - 18}px, ${currentPos.current.y - 18}px) scale(${currentPos.current.scale})`;
			}

			// Arrow square — slower drift, offset bottom-right
			arrowPos.current.x = lerp(arrowPos.current.x, mousePos.current.x + 18, 0.08);
			arrowPos.current.y = lerp(arrowPos.current.y, mousePos.current.y + 18, 0.08);

			// Label pill — even slower drift, offset further right
			labelPos.current.x = lerp(labelPos.current.x, mousePos.current.x + 42, 0.06);
			labelPos.current.y = lerp(labelPos.current.y, mousePos.current.y + 20, 0.06);

			// Smooth opacity for card followers
			const targetOpacity = isOverCardRef.current ? 1 : 0;
			cardOpacity.current = lerp(cardOpacity.current, targetOpacity, 0.1);

			if (arrowRef.current) {
				arrowRef.current.style.transform = `translate(${arrowPos.current.x}px, ${arrowPos.current.y}px) scale(${cardOpacity.current})`;
				arrowRef.current.style.opacity = String(cardOpacity.current);
			}
			if (labelRef.current) {
				labelRef.current.style.transform = `translate(${labelPos.current.x}px, ${labelPos.current.y}px) scale(${cardOpacity.current})`;
				labelRef.current.style.opacity = String(cardOpacity.current);
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
					width: 36,
					height: 36,
					pointerEvents: "none",
					zIndex: 99999,
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					gap: isHovering ? "16px" : "10px",
					fontFamily: "ui-monospace, 'Geist Mono', monospace",
					fontSize: "21px",
					fontWeight: "bold",
					color: "#ffffff",
					textShadow: isHovering ? "0 0 8px rgba(255,255,255,0.6)" : "none",
					mixBlendMode: "difference",
					transition: "gap 0.3s cubic-bezier(0.16, 1, 0.3, 1), text-shadow 0.3s ease",
					willChange: "transform, gap",
				}}
			>
				<span style={{ transform: "translateY(-1px)", zIndex: 2 }}>[</span>
				<div style={{ position: "absolute", top: "50%", left: "50%", width: isHovering ? "24px" : "16px", height: isHovering ? "24px" : "16px", backgroundColor: "rgba(255,255,255,0.15)", backdropFilter: "blur(8px)", WebkitBackdropFilter: "blur(8px)", border: "1px solid rgba(255,255,255,0.25)", transform: "translate(-50%, -50%)", transition: "all 0.3s cubic-bezier(0.16, 1, 0.3, 1)", zIndex: 1, pointerEvents: "none" }} />
				<span style={{ transform: "translateY(-1px)", zIndex: 2 }}>]</span>
			</div>

			{/* Drifting arrow square — appears on card/button hover */}
			<div
				ref={arrowRef}
				style={{
					position: "fixed",
					top: 0,
					left: 0,
					width: 28,
					height: 28,
					background: "#fff",
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					pointerEvents: "none",
					zIndex: 99998,
					opacity: 0,
					willChange: "transform, opacity",
				}}
			>
				<svg viewBox="0 0 10 10" fill="none" width={12} height={12} style={{ transform: "rotate(-45deg)" }}>
					<path d="M3 8L7 4L3 0L4.5 0L8.5 4L4.5 8Z" fill="#000" />
					<path d="M0 3.4h6v1.2H0z" fill="#000" />
				</svg>
			</div>

			{/* VIEW PROJECT pill — drifts further right */}
			<div
				ref={labelRef}
				style={{
					position: "fixed",
					top: 0,
					left: 0,
					background: "#fff",
					padding: "5px 12px",
					pointerEvents: "none",
					zIndex: 99998,
					opacity: 0,
					willChange: "transform, opacity",
					whiteSpace: "nowrap",
				}}
			>
				<span style={{
					fontFamily: "var(--font-mono, monospace)",
					fontSize: 10,
					fontWeight: 600,
					letterSpacing: "0.1em",
					color: "#000",
					textTransform: "uppercase",
				}}>
					view
				</span>
			</div>
		</>
	);
}
