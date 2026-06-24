"use client";

import { ReactLenis, useLenis } from "lenis/react";
import { ReactNode, useEffect } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

function ScrollTriggerBridge() {
	useLenis(() => {
		ScrollTrigger.update();
	});
	return null;
}

export function SmoothScroller({ children }: { children: ReactNode }) {
	useEffect(() => {
		gsap.registerPlugin(ScrollTrigger);
	}, []);

	return (
		<ReactLenis root options={{ lerp: 0.08, duration: 1.2, smoothWheel: true }}>
			<ScrollTriggerBridge />
			{children}
		</ReactLenis>
	);
}
