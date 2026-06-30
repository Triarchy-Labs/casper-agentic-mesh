"use client";

import { useEffect, useRef } from "react";

/**
 * Living carbon-fibre fabric background. Matte woven twill (dark racing red)
 * that parallaxes with the mouse, tilts subtly in 3D, catches a soft light where
 * the cursor is, and gains depth as you scroll — the material is alive, like the
 * old fluid was, but true to a real fabric instead of jelly.
 *
 * Pure CSS-vars + rAF lerp (no WebGL), so it is light and cannot crash the page.
 */
export function CarbonFabric() {
	const stage = useRef<HTMLDivElement>(null);
	const weave = useRef<HTMLDivElement>(null);
	const sheen = useRef<HTMLDivElement>(null);

	useEffect(() => {
		const el = stage.current;
		if (!el) return;

		// targets (t) and smoothed (s) values
		let tpx = 0, tpy = 0, spx = 0, spy = 0;
		let tmx = 50, tmy = 40, smx = 50, smy = 40;
		let scrolly = 0;
		let raf = 0;

		const onMove = (e: MouseEvent) => {
			tpx = (e.clientX / window.innerWidth) * 2 - 1;
			tpy = (e.clientY / window.innerHeight) * 2 - 1;
			tmx = (e.clientX / window.innerWidth) * 100;
			tmy = (e.clientY / window.innerHeight) * 100;
		};
		const onScroll = () => { scrolly = window.scrollY * 0.04; };

		const loop = () => {
			spx += (tpx - spx) * 0.06;
			spy += (tpy - spy) * 0.06;
			smx += (tmx - smx) * 0.12;
			smy += (tmy - smy) * 0.12;
			if (weave.current) {
				weave.current.style.setProperty("--px", spx.toFixed(4));
				weave.current.style.setProperty("--py", spy.toFixed(4));
				weave.current.style.setProperty("--scrolly", scrolly.toFixed(2));
			}
			if (sheen.current) {
				sheen.current.style.setProperty("--mx", `${smx.toFixed(2)}%`);
				sheen.current.style.setProperty("--my", `${smy.toFixed(2)}%`);
			}
			raf = requestAnimationFrame(loop);
		};

		window.addEventListener("mousemove", onMove, { passive: true });
		window.addEventListener("scroll", onScroll, { passive: true });
		raf = requestAnimationFrame(loop);
		return () => {
			window.removeEventListener("mousemove", onMove);
			window.removeEventListener("scroll", onScroll);
			cancelAnimationFrame(raf);
		};
	}, []);

	return (
		<div ref={stage} className="carbon-stage" aria-hidden>
			<div className="carbon-backlight" />
			<div ref={weave} className="carbon-weave" />
			<div className="carbon-depth" />
			<div ref={sheen} className="carbon-sheen" />
		</div>
	);
}
