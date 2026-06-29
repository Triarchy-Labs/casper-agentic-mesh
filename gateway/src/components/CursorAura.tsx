"use client";

import { useEffect, useRef } from "react";

/**
 * Frontier custom cursor: a sharp neon dot with a lerping aura ring that
 * swells over interactive elements. Screen-blended red — racing DNA.
 * No-ops on touch devices (the CSS only hides the native cursor on fine pointers).
 */
export function CursorAura() {
	const dot = useRef<HTMLDivElement>(null);
	const ring = useRef<HTMLDivElement>(null);

	useEffect(() => {
		if (!window.matchMedia("(hover: hover) and (pointer: fine)").matches) return;

		let mx = window.innerWidth / 2, my = window.innerHeight / 2;
		let rx = mx, ry = my;
		let raf = 0;

		const onMove = (e: MouseEvent) => {
			mx = e.clientX; my = e.clientY;
			if (dot.current) dot.current.style.transform = `translate(${mx}px, ${my}px) translate(-50%, -50%)`;
			const hot = (e.target as HTMLElement)?.closest("a, button, [role='button'], textarea, input");
			ring.current?.classList.toggle("is-hot", !!hot);
		};

		const loop = () => {
			rx += (mx - rx) * 0.18;
			ry += (my - ry) * 0.18;
			if (ring.current) ring.current.style.transform = `translate(${rx}px, ${ry}px) translate(-50%, -50%)`;
			raf = requestAnimationFrame(loop);
		};

		window.addEventListener("mousemove", onMove);
		raf = requestAnimationFrame(loop);
		return () => { window.removeEventListener("mousemove", onMove); cancelAnimationFrame(raf); };
	}, []);

	return (
		<>
			<div ref={ring} className="cursor-ring" aria-hidden />
			<div ref={dot} className="cursor-dot" aria-hidden />
		</>
	);
}
