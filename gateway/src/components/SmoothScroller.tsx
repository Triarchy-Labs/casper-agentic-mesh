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
				// produx uses GSAP ScrollSmoother (time-based glide). We match the feel with Lenis in
				// DURATION mode (not lerp — mixing both was a no-op conflict): every scroll eases out
				// over ~1.2s with easeOutExpo, giving one consistent, soft glide across the WHOLE site,
				// so it never drops into "dry" native scroll after the pinned mosaic.
				duration: 1.2,
				easing: (t: number) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
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
