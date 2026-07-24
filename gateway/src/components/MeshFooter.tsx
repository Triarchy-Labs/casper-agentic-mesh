"use client";
import { useEffect, useRef, useState } from "react";
import gsap from "gsap";

// produx footer skeleton (from their static HTML), fused with our content:
// - grid-cols-12: Menu/ (roller [ ] links) · Settlement/ (their Headquarters/) · Proof/ (their
//   Newsletter/ slot) · socials/ (square buttons)
// - the big-logo slot (their mt-[16.6vh] will-change block) carries the CASPER girl banner + a
//   giant CASPER wordmark (TRIARCHY already owns the header — the footer anchors the chain)
// - bottom bar: links + © line
// - reveal: .footer-content slides up from under the page as the footer scrolls in (their
//   will-change-transform wrapper), driven off live rects (pin-spacer-race immune).
const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const PKG = "https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d";

function Roller({ label, href, external }: { label: string; href: string; external?: boolean }) {
	return (
		<a
			href={href}
			{...(external ? { target: "_blank", rel: "noopener noreferrer" } : {})}
			className="group relative flex w-fit cursor-pointer items-center font-light uppercase text-[0.97vw] max-lg:text-[1.36vw] max-sm:text-[3.2vw]"
			style={{ fontFamily: "var(--font-mono)" }}
		>
			<span>[</span>
			<span className="relative flex flex-col justify-center overflow-hidden">
				<span className="block px-1 transition-transform duration-300 ease-out group-hover:-translate-y-[120%]">{label}</span>
				<span className="absolute left-0 block translate-y-[120%] px-1 transition-transform duration-300 ease-out group-hover:translate-y-0 text-[var(--red-700)]">
					{label}
				</span>
			</span>
			<span>]</span>
		</a>
	);
}

function SocialSquare({ glyph, href, title }: { glyph: string; href: string; title: string }) {
	return (
		<a
			href={href}
			target="_blank"
			rel="noopener noreferrer"
			title={title}
			className="group relative inline-flex aspect-square h-[3.2vw] max-lg:h-[5vw] max-sm:h-[10vw] items-center justify-center overflow-hidden border border-white/15 bg-white/[0.04]"
		>
			{/* produx Navbar-Social-Link: label rolls out, red fill sweeps in */}
			<span className="absolute inset-0 scale-150 bg-[var(--red-700)] transition-[clip-path] duration-400 ease-out" style={{ clipPath: "circle(0% at 100% 100%)" }} aria-hidden />
			<span className="relative z-10 font-light text-[0.9vw] max-lg:text-[1.3vw] max-sm:text-[3vw] tracking-[0.1em] text-white/85 transition-colors duration-300 group-hover:text-white" style={{ fontFamily: "var(--font-mono)" }}>
				{glyph}
			</span>
			<style jsx>{`
				a:hover span[aria-hidden] { clip-path: circle(150% at 100% 100%) !important; }
			`}</style>
		</a>
	);
}

export function MeshFooter() {
	const footRef = useRef<HTMLElement>(null);
	const contentRef = useRef<HTMLDivElement>(null);
	const vidRef = useRef<HTMLVideoElement>(null);
	const bannerRef = useRef<HTMLDivElement>(null);
	const [playing, setPlaying] = useState(false);

	// Lazy cinematic banner: preload="none" keeps the network silent until the banner actually
	// enters the viewport — only then play() (which triggers the download). Leaving the section
	// pauses playback, so an idle tab/page never burns cycles on an off-screen loop.
	useEffect(() => {
		const vid = vidRef.current, banner = bannerRef.current;
		if (!vid || !banner) return;
		const onPlaying = () => setPlaying(true);
		vid.addEventListener("playing", onPlaying);
		const io = new IntersectionObserver(
			([e]) => { if (e.isIntersecting) vid.play().catch(() => {}); else vid.pause(); },
			{ threshold: 0.12 }
		);
		io.observe(banner);
		return () => { vid.removeEventListener("playing", onPlaying); io.disconnect(); };
	}, []);

	useEffect(() => {
		const foot = footRef.current;
		const content = contentRef.current;
		if (!foot || !content) return;
		// slide-from-under reveal (their .footer-content will-change-transform): as the footer
		// enters, the content catches up from -38% — reads as the footer emerging from beneath
		// the page. Live rects, same pattern as the cards (immune to pin-spacer trigger races).
		const onScroll = () => {
			const r = foot.getBoundingClientRect();
			const vh = window.innerHeight;
			if (r.top > vh + 80) return;
			const span = Math.min(r.height, vh * 0.9);
			const p = Math.max(0, Math.min(1, (vh - r.top) / span));
			gsap.set(content, { yPercent: (1 - p) * -32 });
		};
		window.addEventListener("scroll", onScroll, { passive: true });
		onScroll();
		return () => window.removeEventListener("scroll", onScroll);
	}, []);

	return (
		<footer
			ref={footRef}
			className="relative w-full overflow-hidden px-[5.5vw] pt-[11.7vh] pb-[3.9vh] max-lg:px-[4.10vw] max-lg:pt-[9vh] max-sm:px-[5.97vw] z-10"
			style={{ backgroundColor: "#0a0508" }}
		>
			<div ref={contentRef} className="footer-content will-change-transform">
				{/* ---- columns (produx grid) ---- */}
				<div className="grid grid-cols-12 gap-y-[4vh] max-lg:grid-cols-8">
					{/* Menu/ */}
					<div className="col-span-2 flex flex-col gap-[1.67vw] max-sm:col-span-8">
						<h5 className="label-13-mono text-white/50">Menu/</h5>
						<div className="flex flex-col items-start gap-[1.2vw] max-sm:gap-[2vh]">
							<Roller label="Home" href="/" />
							<Roller label="Bounties" href="/bounties" />
							<Roller label="Dashboard" href="/dashboard" />
						</div>
					</div>

					{/* Settlement/ (their Headquarters/) */}
					<div className="col-span-2 col-start-3 flex flex-col gap-[1.67vw] max-lg:col-span-3 max-sm:col-span-8">
						<h5 className="label-13-mono text-white/50">Settlement/</h5>
						<div className="flex flex-col gap-[0.8vh] label-13-mono text-[var(--gray-800)]">
							<p>Casper Testnet — live</p>
							<p>Rust / WASM · x402</p>
							<Roller label="Contract" href={PKG} external />
						</div>
					</div>

					{/* Proof/ (their Newsletter/ slot) */}
					<div className="col-span-4 col-start-7 flex flex-col gap-[1.67vw] max-lg:col-span-3 max-lg:col-start-4 max-sm:col-span-8">
						<h5 className="label-13-mono text-white/50">Proof/</h5>
						<p className="label-13-mono text-[var(--gray-800)]" style={{ lineHeight: 1.7 }}>
							Nothing here is a promise. Every claim on this page opens its own on-chain proof.
						</p>
						<div className="flex gap-[1.4vw] flex-wrap">
							<Roller label="Deployments" href={`${REPO}/blob/main/DEPLOYMENTS.md`} external />
							<Roller label="Judge proof" href={`${REPO}/blob/main/JUDGE_PROOF.md`} external />
						</div>
					</div>

					{/* socials/ */}
					<div className="col-span-2 flex flex-col gap-[1.67vw] justify-self-end max-lg:col-span-2 max-sm:col-span-8 max-sm:justify-self-start">
						<h5 className="label-13-mono text-white/50">Links/</h5>
						<div className="flex gap-[0.83vw] max-sm:gap-[2vw]">
							<SocialSquare glyph="GH" href={REPO} title="GitHub" />
							<SocialSquare glyph="DH" href="https://dorahacks.io/hackathon/casper" title="DoraHacks" />
							<SocialSquare glyph="CS" href="https://casper.network" title="casper.network" />
						</div>
					</div>
				</div>

				{/* ---- the big-logo slot: crimson megacity LIVE banner ---- */}
				<div ref={bannerRef} className="relative mt-[12vh] w-full overflow-hidden max-lg:mt-[9vh] max-sm:mt-[5.14vh]">
					<div className="relative w-full aspect-[16/9]">
						{/* poster paints instantly (412KB), the 4s loop cross-fades over it once playing */}
						<img
							src="/footer-crimson-poster.jpg"
							alt="CASPER — the crimson megacity"
							className="absolute left-1/2 top-1/2 h-[116%] w-[116%] -translate-x-1/2 -translate-y-1/2 object-cover"
						/>
						<video
							ref={vidRef}
							muted
							loop
							playsInline
							preload="none"
							poster="/footer-crimson-poster.jpg"
							className="absolute left-1/2 top-1/2 h-[116%] w-[116%] -translate-x-1/2 -translate-y-1/2 object-cover"
							style={{ opacity: playing ? 1 : 0, transition: "opacity 1.4s ease" }}
						>
							<source src="/footer-crimson.webm" type="video/webm" />
							<source src="/footer-crimson.mp4" type="video/mp4" />
						</video>
					</div>
					{/* cinematic vignette: edges sink into the footer's black, bottom melts into it */}
					<div
						className="pointer-events-none absolute inset-0"
						style={{
							background:
								"radial-gradient(ellipse at 50% 42%, transparent 50%, rgba(0,0,0,0.55) 100%), linear-gradient(to top, rgba(10,5,8,0.9), rgba(10,5,8,0.25) 14%, transparent 28%), linear-gradient(to bottom, rgba(10,5,8,0.4), transparent 12%)",
						}}
					/>
				</div>

				{/* ---- compact bottom: ghost wordmark LEFT, everything else stacked RIGHT under the photo ---- */}
				<div className="mt-[2.5vh] flex items-end justify-between gap-[3vw] flex-wrap">
					{/* ghost wordmark: 95% transparent, wakes +5% on hover (user spec) */}
					<div
						className="uppercase leading-[0.85] text-white select-none opacity-[0.05] hover:opacity-[0.10] transition-opacity duration-500 cursor-default"
						style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(64px, 11vw, 220px)", fontWeight: 400, letterSpacing: "0.02em" }}
					>
						C<span className="text-[var(--red-700)]">Λ</span>SPER
					</div>
					<div className="flex flex-col items-end gap-[1.4vh] pb-[0.8vh] max-sm:items-start max-sm:w-full">
						<p className="label-13-mono text-white/40">forged by TRIARCHY // the mesh settles here</p>
						<div className="flex items-center gap-3 label-12-mono text-[var(--gray-600)] flex-wrap justify-end max-sm:justify-start">
							<span className="text-[var(--red-700)]">///</span>
							<span>REV 2.6</span>
							<span className="text-[var(--gray-500)]">·</span>
							<span>UNIT / MESH-01</span>
							<span className="text-[var(--gray-500)]">·</span>
							<span>BUILD 2026.07</span>
							<span className="text-[var(--gray-500)]">·</span>
							<span>INSTANTIATED ON CASPER NETWORK</span>
						</div>
						<div className="flex gap-[1.69vw] flex-wrap justify-end max-sm:justify-start">
							<Roller label="x402 spec" href="https://www.x402.org/" external />
							<Roller label="casper.network" href="https://casper.network" external />
							<Roller label="Odra framework" href="https://odra.dev" external />
						</div>
						<p className="label-13-mono text-white/40">© 2026 TRIARCHY LABS — ALL VERDICTS FINAL.</p>
					</div>
				</div>
			</div>
		</footer>
	);
}
