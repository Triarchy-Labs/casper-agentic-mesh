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
		<ReactLenis
			root
			autoRaf={false}
			options={{
				// Keep the site OILY (low lerp = long, soft inertia glide — the feel we want everywhere).
				// The crystal's coast-tick is fixed at the crystal itself (velocity-gated frame), NOT by
				// making the whole site crisper. 0.05 = a touch more glide than the previous 0.06.
				lerp: 0.05,
				smoothWheel: true,
				wheelMultiplier: 1,
				touchMultiplier: 1.6,
				syncTouch: true,
			}}
		>
			<ScrollTriggerBridge />
			{children}
		</ReactLenis>
	);
}
