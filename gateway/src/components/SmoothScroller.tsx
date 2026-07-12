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
				// lerp = how fast the scroll catches up to its target. LOW (floaty) = long inertia coast
				// after you stop = the crystal's frames tick through it visibly. 0.1 shortens that coast so
				// the settle is quick (ticks blur into one smooth stop). Site-wide feel lever; tune here.
				lerp: 0.1,
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
