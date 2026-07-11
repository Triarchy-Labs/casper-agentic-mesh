"use client";

import { ReactLenis, useLenis } from "lenis/react";
import { ReactNode, useEffect } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

// Canonical Lenis <-> GSAP integration: Lenis is driven by the GSAP ticker (single clock),
// and ScrollTrigger updates on every Lenis scroll. This removes the RAF desync that makes
// scrubbed / pinned animations feel jittery or "hyperspeed".
function ScrollTriggerBridge() {
	const lenis = useLenis(() => ScrollTrigger.update());

	useEffect(() => {
		if (!lenis) return;
		const raf = (time: number) => lenis.raf(time * 1000);
		gsap.ticker.add(raf);
		gsap.ticker.lagSmoothing(0);
		return () => {
			gsap.ticker.remove(raf);
		};
	}, [lenis]);

	return null;
}

export function SmoothScroller({ children }: { children: ReactNode }) {
	useEffect(() => {
		gsap.registerPlugin(ScrollTrigger);
	}, []);

	return (
		<ReactLenis root autoRaf={false} options={{ lerp: 0.08, duration: 1.2, smoothWheel: true }}>
			<ScrollTriggerBridge />
			{children}
		</ReactLenis>
	);
}
