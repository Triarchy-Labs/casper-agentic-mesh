"use client";
import { useEffect, useRef } from "react";

// HoverReel — the living-card mechanic: the still art holds; hover lingers ~0.9s, then the
// photo cross-fades into its animated take (the video generated FROM that exact still).
// Leave = fade back to the still, rewind so every hover starts from frame 0.
// Costs nothing until hovered: preload="none", play() triggers the download.
// prefers-reduced-motion users keep the stills — the reel never engages.
export function HoverReel({ name }: { name: string }) {
	const ref = useRef<HTMLVideoElement>(null);

	useEffect(() => {
		const v = ref.current;
		if (!v) return;
		if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
		const host = v.closest(".group") as HTMLElement | null;
		if (!host) return;

		let timer: ReturnType<typeof setTimeout> | null = null;
		const enter = () => {
			timer = setTimeout(() => {
				v.play().catch(() => {});
				v.classList.add("hr-on");
			}, 900);
		};
		const leave = () => {
			if (timer) clearTimeout(timer);
			v.classList.remove("hr-on");
			setTimeout(() => {
				if (!v.classList.contains("hr-on")) {
					v.pause();
					try { v.currentTime = 0; } catch { /* not loaded yet */ }
				}
			}, 650);
		};
		host.addEventListener("mouseenter", enter);
		host.addEventListener("mouseleave", leave);
		return () => {
			if (timer) clearTimeout(timer);
			host.removeEventListener("mouseenter", enter);
			host.removeEventListener("mouseleave", leave);
		};
	}, []);

	return (
		<video
			ref={ref}
			muted
			loop
			playsInline
			preload="none"
			className="hover-reel absolute inset-0 h-full w-full object-cover"
			aria-hidden
		>
			<source src={`/reels/${name}.webm`} type="video/webm" />
			<source src={`/reels/${name}.mp4`} type="video/mp4" />
		</video>
	);
}
