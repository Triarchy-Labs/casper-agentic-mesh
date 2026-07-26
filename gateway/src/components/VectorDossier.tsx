"use client";
import { motion } from "framer-motion";
import Image from "next/image";
import { useEffect, useRef, useState, useCallback } from "react";
import gsap from "gsap";
import Lenis from "lenis";
import { CustomEase } from "gsap/CustomEase";

gsap.registerPlugin(CustomEase);

// produx project-detail mechanics, faithfully:
// 1) the cursor VIEW pill lives in CustomCursor (site-wide, drifting followers);
// 2) click → the card's art MORPHS into the detail hero (FLIP: fixed clone animates from the
//    card rect to the hero rect on natureSway) — the in-page equivalent of their
//    startViewTransition shared-element morph;
// 3) their Art-Direction section rhythm below, carrying our verifiable content;
// 4) [ CLOSE × ] / ESC reverses the morph back into the card.

const EX = "https://testnet.cspr.live";
const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const glide: [number, number, number, number] = [0.1, 0.4, 0.15, 1];

export type DossierOpen = { slug: string; rect: { top: number; left: number; width: number; height: number } };

type Proof = { label: string; href: string };
type Vector = {
	slug: string;
	index: string;
	kicker: string;
	title: string;
	art: string;
	tags: string[];
	lede: string;
	how: string[];
	synergy: string;
	proof: Proof[];
	frames: number;
	marks: { tag: string; note: string }[];
	stats: { v: string; k: string }[];
	quote: [string, string, string]; // [before, red word, after]
	media: { src: string; cap: string }[];
};

export const VECTORS: Vector[] = [
	{
		slug: "escrow",
		index: "01",
		kicker: "vector alpha",
		title: "Autonomous Escrow",
		art: "/vector_escrow.jpeg",
		tags: ["Escrow", "Proxy", "Deterministic"],
		lede: "CSPR is locked in the contract's own purse. The only ways out are release → the registered hunter, or refund → the creator. Nothing else can move it.",
		how: [
			"create_bounty locks funds via session code (deposit-proxy) — a stored contract cannot spend a caller's main purse, so escrow is funded the canonical Casper way.",
			"release_bounty and refund_bounty are the contract's ONLY money-moving entry points.",
			"Both terminal paths are proven on testnet by on-chain dictionary read-back — the full state machine, not just tx success.",
		],
		synergy: "This is where enforcement physically lives. The Tribunal only decides between these two paths — a verdict, sane or hallucinated, can trigger nothing else. Trust is written here, in code.",
		proof: [
			{ label: "contract package ↗", href: `${EX}/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d` },
			{ label: "release_bounty tx ↗", href: `${EX}/transaction/1ea27a03a072b0db1f8b5f4cf176364eec9ef50cb396bafb9f56829c21204f14` },
			{ label: "refund_bounty tx ↗", href: `${EX}/transaction/895eb5531398c44a85554c11c622d3f528ef73ac9e541619f163ec392e120d87` },
		],
		frames: 2,
		marks: [
			{ tag: "Escrow", note: "Your CSPR sits in the contract's own purse — not in anyone's wallet, not in ours." },
			{ tag: "Proxy", note: "Funding runs through session code — the canonical Casper way into a contract purse." },
			{ tag: "Deterministic", note: "Two exits exist. No verdict, sane or insane, can invent a third." },
		],
		stats: [
			{ v: "2", k: "money paths — release or refund" },
			{ v: "10 CSPR", k: "settled in the live on-chain demo" },
			{ v: "0", k: "other exits, by construction" },
		],
		quote: ["Trust isn't promised here. ", "It's compiled", "."],
		media: [
			{ src: "/cards/kpi-volume.webp", cap: "the stake, crystallized — value the contract holds until the verdict" },
			{ src: "/cards/agent-alpha.webp", cap: "the triarchy mark — three powers, one law" },
		],
	},
	{
		slug: "omni-mesh",
		index: "02",
		kicker: "the organism",
		title: "The Omni-Mesh",
		art: "/card-omni-mesh.jpg",
		tags: ["Gateway", "A2A", "MCP"],
		lede: "The five vectors as one economic OS for autonomous agents — discover, pay, work, get judged, get paid, all under one roof.",
		how: [
			"A Next.js gateway fronts the contracts and the swarm; any agent discovers its capabilities at GET /api/mcp.",
			"Agents onboard with the published Casper Agent Skill — no bespoke integration, just the manifest and a tx hash.",
			"Every other vector plugs into this spine: escrow, oracle, tribunal, x402 payments.",
		],
		synergy: "The mesh is what turns five contracts and agents into a market. It is the surface a judge — or a rival agent — actually touches.",
		proof: [
			{ label: "MCP manifest ↗", href: "https://casper-agentic-mesh.vercel.app/api/mcp" },
			{ label: "Casper Agent Skill ↗", href: `${REPO}/blob/main/CASPER_AGENT_SKILL.md` },
		],
		frames: 3,
		marks: [
			{ tag: "Gateway", note: "One front door for humans and machines — the same endpoints serve both." },
			{ tag: "A2A", note: "Agents hire agents: the bounty flow speaks bot-to-bot, no human in the loop required." },
			{ tag: "MCP", note: "A manifest any AI can read and act on. No docs, no onboarding call." },
		],
		stats: [
			{ v: "5", k: "vectors under one roof" },
			{ v: "1", k: "manifest to join — GET /api/mcp" },
			{ v: "30s", k: "from curl to your first answer" },
		],
		quote: ["Five vectors. ", "One organism", "."],
		media: [
			{ src: "/cards/telemetry-eyes.webp", cap: "the mesh watches its own — every agent under the same eyes" },
			{ src: "/cards/terminal-rooftop.webp", cap: "the city the agents work — settlement never sleeps" },
		],
	},
	{
		slug: "oracle",
		index: "03",
		kicker: "vector beta",
		title: "RWA Risk Oracle",
		art: "/vector_oracle.jpeg",
		tags: ["Oracle", "Price Feed", "Reputation"],
		lede: "A live on-chain data feed with agent identity and reputation that accrues from real work — not opinions.",
		how: [
			"The oracle contract stores readings, agent identity and an append-only event log.",
			"The rwa-oracle agent posts a real CSPR/USD price on-chain; read-back confirms the value and accruing reputation.",
			"Reputation is minted from realized on-chain outcomes, so it cannot be faked or self-rated.",
		],
		synergy: "It prices RWA-pegged bounties for Escrow and anchors every Tribunal verdict — the mesh's memory and its sense of value.",
		proof: [
			{ label: "oracle package ↗", href: `${EX}/contract-package/16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2` },
			{ label: "live CSPR/USD feed tx ↗", href: `${EX}/transaction/da7ac22bc69c801a3600d43d408a29c85170f9205d224c3345b3f482d1949300` },
		],
		frames: 2,
		marks: [
			{ tag: "Oracle", note: "A real data feed written to the chain — not an API promise you can't audit." },
			{ tag: "Price Feed", note: "CSPR-USD, read back live on this very site. Refresh and watch it." },
			{ tag: "Reputation", note: "Minted from settled outcomes only. No self-rating. No purchase." },
		],
		stats: [
			{ v: "live", k: "CSPR-USD feed, open on cspr.live" },
			{ v: "7", k: "reputation minted from real outcomes" },
			{ v: "1", k: "append-only event log — history can't be rewritten" },
		],
		quote: ["Reputation you can't buy. ", "Only earn", ", on the record."],
		media: [
			{ src: "/cards/tower-control.webp", cap: "the reading room — every screen is a corner of the world" },
			{ src: "/cards/kpi-efficiency.webp", cap: "unit 110 weighing a reading — machines checking machines" },
		],
	},
	{
		slug: "tribunal",
		index: "04",
		kicker: "vector gamma",
		title: "Agent Tribunal",
		art: "/card-tribunal.jpg",
		tags: ["Adversarial", "5 LLMs", "Chief Judge"],
		lede: "An adversarial court of five real LLMs — prosecutor, defender, three jurors, chief judge — that rules on submitted work and moves CSPR on-chain.",
		how: [
			"Prosecutor and defender argue; three jurors of diverse models vote; a chief judge rules.",
			"Both verdict paths are exercised live on testnet; --dry-run deliberates without spending.",
			"Fault-tolerant: a partial bench rules 'indicative, not fully precise'; all agents down → 'functions frozen, no funds moved'.",
		],
		synergy: "Its verdict flows straight into Escrow's two bounded paths and is anchored on the Oracle. The LLM argues; the contract enforces.",
		proof: [
			{ label: "REJECT → refund tx ↗", href: `${EX}/transaction/4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03` },
			{ label: "APPROVE → release tx ↗", href: `${EX}/transaction/702132683a246c1e07e7c49f0e403b680d85b7114b8ec25772af5991a959c375` },
		],
		frames: 3,
		marks: [
			{ tag: "Adversarial", note: "A prosecutor attacks every submission before it earns a single coin." },
			{ tag: "5 LLMs", note: "Five real models from different vendors, arguing in the open — transcripts included." },
			{ tag: "Chief Judge", note: "One ruling, bounded by the contract: release or refund. Nothing else exists." },
		],
		stats: [
			{ v: "5", k: "real models on the bench" },
			{ v: "3", k: "jurors, deliberately diverse vendors" },
			{ v: "2", k: "verdict paths — both proven on-chain" },
		],
		quote: ["The court argues. ", "The contract obeys no one's eloquence", "."],
		media: [
			{ src: "/cards/kpi-quests.webp", cap: "the prosecution never sleeps — every claim faces the dragon" },
			{ src: "/cards/tribunal-arena.webp", cap: "the arena — verdicts land over a living city" },
		],
	},
	{
		slug: "x402",
		index: "05",
		kicker: "vector delta",
		title: "x402 Payment Layer",
		art: "/card-x402.jpg",
		tags: ["HTTP 402", "Pay-per-call", "Replay Guard"],
		lede: "HTTP 402-native, pay-per-call settlement in real CSPR — no subscriptions, no API keys, just value for value.",
		how: [
			"POST /api/hire or /api/orchestrator with an x-l402-txhash header; no hash → an honest 402 Payment Required.",
			"Payment is validated against the ledger (recipient + amount) with a single-use replay guard, 5-minute TTL.",
			"Fabricated or reused hashes are rejected — we never trust a string.",
		],
		synergy: "It is how an agent pays to enter the mesh, and it feeds Escrow. The x402 protocol working as intended is a 402, not an error.",
		proof: [
			{ label: "hire route ↗", href: `${REPO}/blob/main/gateway/src/app/api/hire/route.ts` },
			{ label: "Casper Agent Skill ↗", href: `${REPO}/blob/main/CASPER_AGENT_SKILL.md` },
		],
		frames: 2,
		marks: [
			{ tag: "HTTP 402", note: "The status code the web reserved for payment since 1997. We finally use it." },
			{ tag: "Pay-per-call", note: "No subscriptions, no API keys. One payment, one job, one receipt." },
			{ tag: "Replay Guard", note: "Every hash is single-use. Fabrications die at the ledger, not at our word." },
		],
		stats: [
			{ v: "402", k: "the honest refusal — try it live" },
			{ v: "1", k: "use per payment hash, ever" },
			{ v: "5 min", k: "receipt TTL — stale proofs expire" },
		],
		quote: ["No keys. No subscriptions. ", "Value for value", ", verified."],
		media: [
			{ src: "/cards/agent-aegis.webp", cap: "x402, engraved — payment as a blade's oath" },
			{ src: "/cards/agent-sarcophagus.webp", cap: "the toll gate — pay to pass, provably" },
		],
	},
];

/* ────────────────────────────────────────────────────────────────────────────
   DOSSIER — FLIP morph open/close + produx section rhythm.
   ──────────────────────────────────────────────────────────────────────────── */
function Section({ n, title, children }: { n: string; title: string; children: React.ReactNode }) {
	return (
		<motion.div
			initial={{ opacity: 0, y: 26 }}
			whileInView={{ opacity: 1, y: 0 }}
			viewport={{ once: true, margin: "-12% 0px" }}
			transition={{ duration: 0.9, ease: glide }}
			className="grid grid-cols-12 gap-x-[1.39vw] gap-y-4 border-t border-white/10 pt-[5vh]"
		>
			<div className="col-span-12 md:col-span-3 flex items-baseline gap-3">
				<span className="label-13-mono text-[var(--red-700)]">{n}</span>
				<h3 className="label-14-mono text-white/50 uppercase tracking-[0.14em]">{title}</h3>
			</div>
			<div className="col-span-12 md:col-span-8 md:col-start-5">{children}</div>
		</motion.div>
	);
}

export function VectorDossier({ open, onClose }: { open: DossierOpen | null; onClose: () => void }) {
	const v = open ? VECTORS.find((x) => x.slug === open.slug) || null : null;
	const [phase, setPhase] = useState<"idle" | "morphing" | "open" | "closing">("idle");
	const [hintGone, setHintGone] = useState(false);
	const overlayRef = useRef<HTMLDivElement>(null);
	const scrollerRef = useRef<HTMLDivElement>(null);
	const contentRef = useRef<HTMLDivElement>(null);
	const heroRef = useRef<HTMLDivElement>(null);
	const cloneRef = useRef<HTMLDivElement>(null);
	const srcRect = useRef<DossierOpen["rect"] | null>(null);

	// OPEN: set phase + lock body. All measuring/animating happens in the phase effect below,
	// which React guarantees runs AFTER the overlay+clone are committed to the DOM — the old
	// requestAnimationFrame version raced React's commit and could measure a not-yet-mounted
	// clone, leaving an invisible full-screen overlay that swallowed every click (measured live).
	useEffect(() => {
		if (!open || !v) return;
		srcRect.current = open.rect;
		setPhase("morphing");
		const prev = document.body.style.overflow;
		document.body.style.overflow = "hidden";
		document.body.style.cursor = "";
		return () => { document.body.style.overflow = prev; };
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open?.slug]);

	// LOCAL smooth scroll for the dossier: the root Lenis is told to ignore this subtree
	// (data-lenis-prevent), which left the overlay with raw native wheel. A scoped Lenis
	// instance (wrapper/content mode) restores the exact site-wide oil (lerp 0.05) inside,
	// living only while the dossier is open.
	useEffect(() => {
		if (!open) return;
		const wrapper = scrollerRef.current, content = contentRef.current;
		if (!wrapper || !content) return;
		const lenis = new Lenis({ wrapper, content, lerp: 0.05, smoothWheel: true, wheelMultiplier: 1, touchMultiplier: 1.6 });
		setHintGone(false);
		const onScroll = () => { if (wrapper.scrollTop > 60) setHintGone(true); };
		wrapper.addEventListener("scroll", onScroll, { passive: true });
		const raf = (time: number) => lenis.raf(time * 1000);
		gsap.ticker.add(raf);
		return () => { wrapper.removeEventListener("scroll", onScroll); gsap.ticker.remove(raf); lenis.destroy(); };
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [open?.slug]);

	// PHASE: morphing → run the forward morph (DOM is committed by the time effects fire)
	useEffect(() => {
		if (phase !== "morphing") return;
		const clone = cloneRef.current, hero = heroRef.current, ov = overlayRef.current, sc = scrollerRef.current;
		const s = srcRect.current;
		if (!clone || !hero || !ov || !s) { setPhase("open"); return; }
		try { CustomEase.create("natureSway", "M0,0 C0.08,0.494 0.14,1 1,1"); } catch { /* registered */ }
		if (sc) sc.scrollTop = 0;
		const t = hero.getBoundingClientRect();
		gsap.set(clone, { top: s.top, left: s.left, width: s.width, height: s.height, autoAlpha: 1 });
		gsap.to(ov, { opacity: 1, duration: 0.6, ease: "power2.out" });
		const tween = gsap.to(clone, {
			top: t.top, left: t.left, width: t.width, height: t.height,
			duration: 1.25, ease: "power2.inOut",
			onComplete: () => setPhase("open"),
		});
		return () => { tween.kill(); };
	}, [phase]);

	// PHASE: open → hand off from clone to the real hero (no flicker), listen for ESC
	useEffect(() => {
		if (phase !== "open") return;
		const clone = cloneRef.current;
		if (clone) gsap.to(clone, { autoAlpha: 0, duration: 0.15, delay: 0.05 });
		const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setPhase("closing"); };
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [phase]);

	// PHASE: closing → reverse morph (or plain fade when scrolled deep), then unmount
	useEffect(() => {
		if (phase !== "closing") return;
		const clone = cloneRef.current, hero = heroRef.current, ov = overlayRef.current, sc = scrollerRef.current;
		const s = srcRect.current;
		const finish = () => { setPhase("idle"); onClose(); };
		if (!clone || !hero || !ov || !s || !sc) { finish(); return; }
		const hr = hero.getBoundingClientRect();
		const heroVisible = hr.bottom > 0 && hr.top < window.innerHeight;
		if (heroVisible) {
			const t = hero.getBoundingClientRect();
			gsap.set(clone, { top: t.top, left: t.left, width: t.width, height: t.height, autoAlpha: 1 });
			gsap.to(clone, { top: s.top, left: s.left, width: s.width, height: s.height, duration: 1.0, ease: "power2.inOut" });
			gsap.to(ov, { opacity: 0, duration: 0.6, ease: "power2.inOut", delay: 0.25, onComplete: finish });
		} else {
			gsap.to(ov, { opacity: 0, duration: 0.5, ease: "power2.inOut", onComplete: finish });
		}
	}, [phase, onClose]);

	const requestClose = useCallback(() => {
		setPhase((p) => (p === "open" || p === "morphing" ? "closing" : p));
	}, []);

	if (!open || !v) return null;
	const showContent = phase === "open";

	return (
		<div>
			{/* overlay */}
			<div
				ref={overlayRef}
				className="fixed inset-0 z-[9990]"
				style={{ backgroundColor: "#050305", opacity: 0 }}
			>
				<div ref={scrollerRef} data-lenis-prevent className="h-full w-full overflow-y-auto overflow-x-hidden overscroll-contain">
					<div ref={contentRef}>
					{/* ── TEXT HERO — produx /projects layout: kicker + big title LEFT, lede below-left,
					       tags RIGHT; dark block first, image comes after it, full-bleed. Visible from
					       the very start of the morph (like their page nav), not gated on it. ── */}
					<div className="mx-auto w-full max-w-[1500px] px-[5.5vw] pt-[16vh] pb-[8vh] max-sm:pt-[12vh]">
						<motion.div
							initial={{ opacity: 0, y: 26 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 0.9, ease: glide, delay: 0.15 }}
						>
							<span className="label-14-mono text-[var(--red-700)]">{v.index} // {v.kicker}</span>
							<h2 className="mt-3 uppercase leading-[0.95]" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(40px, 6vw, 104px)", fontWeight: 400 }}>
								{v.title}
							</h2>
						</motion.div>
						<motion.div
							initial={{ opacity: 0, y: 20 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 0.9, ease: glide, delay: 0.3 }}
							className="mt-[5vh] flex flex-wrap items-end justify-between gap-6"
						>
							<p className="max-w-[52ch] text-white/80" style={{ fontFamily: "var(--font-DM-mono), var(--font-mono), monospace", fontSize: "clamp(15px, 1.15vw, 22px)", lineHeight: 1.55 }}>
								{v.lede}
							</p>
							<div className="flex gap-2 flex-wrap pb-1">
								{v.tags.map((t) => <span key={t} className="nb-tag">{t}</span>)}
							</div>
						</motion.div>
					</div>

					{/* ── FULL-BLEED IMAGE — the morph target: the card art lands edge-to-edge, like
					       their project photo taking the whole page width. ── */}
					<div ref={heroRef} className="relative w-full aspect-[1.9/1] max-sm:aspect-[1.3/1] overflow-hidden bg-black">
						<Image
							src={v.art}
							alt={v.title}
							fill
							style={{ objectFit: "cover", opacity: showContent ? 1 : 0 }}
							quality={95}
							priority={true}
						/>
					</div>

					<div className="mx-auto w-full max-w-[1500px] px-[5.5vw] pb-[16vh]">
						{showContent && (
							<div className="mt-[7vh] grid grid-cols-1 gap-6 md:grid-cols-3">
								{v.marks.map((m, i) => (
									<motion.div key={m.tag} initial={{ opacity: 0, y: 18 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.7, ease: glide, delay: 0.15 + i * 0.08 }}>
										<p className="label-13-mono text-[var(--red-700)] tracking-[0.16em]">{m.tag}</p>
										<p className="label-13-mono mt-2 leading-[1.7] text-white/60" style={{ textTransform: "none" }}>{m.note}</p>
									</motion.div>
								))}
							</div>
						)}
						{showContent && (
							<div className="mt-[12vh] flex flex-col gap-[10vh]">
								<Section n="/01" title="How it works">
									<div className="flex flex-col gap-[1.2vw]">
										{v.how.map((line, i) => (
											<p key={i} className="label-13-mono text-white/75 leading-[1.7] flex gap-3">
												<span className="text-[var(--red-700)]">→</span>
												<span>{line}</span>
											</p>
										))}
									</div>
								</Section>

								<div className="grid grid-cols-1 gap-8 border-t border-white/10 pt-[5vh] md:grid-cols-3">
									{v.stats.map((st) => (
										<div key={st.k}>
											<p className="leading-none text-white" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(34px, 3.6vw, 64px)", fontWeight: 400 }}>{st.v}</p>
											<p className="label-12-mono mt-3 text-white/45" style={{ textTransform: "none" }}>{st.k}</p>
										</div>
									))}
								</div>

								<motion.p initial={{ opacity: 0, y: 26 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true, margin: "-10% 0px" }} transition={{ duration: 0.9, ease: glide }} className="max-w-[24ch] uppercase leading-[1.08]" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(30px, 3.4vw, 58px)", fontWeight: 400 }}>
									{v.quote[0]}<span className="text-[var(--red-700)]">{v.quote[1]}</span>{v.quote[2]}
								</motion.p>

								<div className="grid grid-cols-1 gap-[1.39vw] md:grid-cols-2">
									{v.media.map((mm) => (
										<motion.figure key={mm.src} initial={{ opacity: 0, y: 24 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true, margin: "-8% 0px" }} transition={{ duration: 0.8, ease: glide }} className="m-0">
											<div className="card-art relative aspect-[1.5/1] overflow-hidden border border-white/10 bg-black">
												<Image src={mm.src} alt={mm.cap} fill style={{ objectFit: "cover" }} quality={95} />
											</div>
											<figcaption className="label-12-mono mt-3 text-white/40" style={{ textTransform: "none" }}>{mm.cap}</figcaption>
										</motion.figure>
									))}
								</div>

								<Section n="/02" title="How it synergizes">
									<p className="text-white/75 leading-[1.6]" style={{ fontFamily: "var(--font-DM-mono), var(--font-mono), monospace", fontSize: "clamp(15px, 1.2vw, 22px)" }}>
										{v.synergy}
									</p>
								</Section>

								<Section n="/03" title="Verify on-chain">
									<div className="flex flex-wrap gap-x-[1.6vw] gap-y-3">
										{v.proof.map((p) => (
											<a key={p.href} href={p.href} target="_blank" rel="noopener noreferrer" className="label-13-mono uppercase tracking-[0.12em] text-white/70 hover:text-[var(--red-700)] transition-colors">
												[ {p.label} ]
											</a>
										))}
									</div>
									<p className="mt-6 label-12-mono text-white/35">Don't trust this panel — open the hashes. Full path: PLAYBOOK.md</p>
								</Section>
							</div>
						)}
					</div>
					</div>
				</div>

				{/* scroll hint — pulses over the full-bleed art, dies on first scroll */}
				<div
					className="pointer-events-none fixed bottom-[4vh] left-1/2 -translate-x-1/2 z-[9998] flex items-center gap-2 transition-opacity duration-700"
					style={{ opacity: phase === "open" && !hintGone ? 0.65 : 0 }}
					aria-hidden
				>
					<span className="scroll-hint-pulse label-12-mono uppercase tracking-[0.3em] text-white/45" style={{ textShadow: "0 1px 8px rgba(0,0,0,0.8)" }}>
						[ scroll <span className="text-[var(--red-700)]/70">↓</span> ]
					</span>
				</div>

				{/* close */}
				<button
					onClick={requestClose}
					aria-label="Close"
					className="group fixed top-[3vh] right-[3vw] z-[10000] flex items-center gap-2 label-13-mono text-white/70 hover:text-white transition-colors"
				>
					<span className="tracking-[0.16em]">[ CLOSE</span>
					<span className="inline-block transition-transform duration-300 group-hover:rotate-90">✕</span>
					<span className="tracking-[0.16em]">]</span>
				</button>
			</div>

			{/* MORPH CLONE — the card art travelling to/from the hero. Always mounted while the
			    dossier exists (phases only animate it), so measuring never races React's commit. */}
			<div
				ref={cloneRef}
				className="card-art fixed z-[9995] overflow-hidden pointer-events-none bg-black"
				style={{ visibility: "hidden" }}
			>
				<Image src={v.art} alt="Morph Clone" fill style={{ objectFit: "cover" }} quality={95} priority={true} />
			</div>
		</div>
	);
}
