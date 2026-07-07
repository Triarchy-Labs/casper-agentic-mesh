"use client";
import { useEffect, useState } from "react";
import { motion, useScroll, useTransform } from "framer-motion";

/**
 * Cinematic darkening layer over the hero: a radial vignette (depth, edges
 * darkest) multiplied onto the bright carbon, ~60% at the top. It fades out as
 * you scroll so full brightness is restored by roughly the 3rd screen.
 * Sits above the background but below the content, so text stays crisp.
 */
export function CinematicDim() {
	const { scrollY } = useScroll();
	const [vh, setVh] = useState(1000);

	useEffect(() => {
		const read = () => setVh(window.innerHeight);
		read();
		window.addEventListener("resize", read);
		return () => window.removeEventListener("resize", read);
	}, []);

	// Full dim at top → half by ~1.3 screens → gone by ~2.7 screens (viewport-relative).
	const opacity = useTransform(scrollY, [0, vh * 1.3, vh * 2.7], [1, 0.5, 0]);

	return (
		<motion.div
			aria-hidden
			style={{
				position: "fixed",
				inset: 0,
				zIndex: 5,
				pointerEvents: "none",
				opacity,
				mixBlendMode: "multiply",
				background:
					"radial-gradient(ellipse at 50% 40%, rgba(0,0,0,0.30) 0%, rgba(0,0,0,0.55) 52%, rgba(0,0,0,0.86) 100%)",
			}}
		/>
	);
}
