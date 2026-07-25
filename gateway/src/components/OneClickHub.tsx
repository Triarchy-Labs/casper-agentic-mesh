"use client";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";

// ONE CLICK — the navigation hub that cures "hard to navigate" (judge feedback) in one gesture.
// A capsule floats bottom-center once the hero is passed; clicking it unfolds a Lama-Lama-style
// panel: plain rows to every important place, the Casper Agent Skill scroll, and hard links.
// Pure navigation layer — touches nothing else on the page.

const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const glide: [number, number, number, number] = [0.22, 1, 0.36, 1];

const ROWS: { label: string; sub: string; href?: string; anchor?: string }[] = [
	{ label: "How it works", sub: "five vectors · the cards", anchor: ".focus-cards" },
	{ label: "Judge playbook", sub: "verify everything · 10 min", href: `${REPO}/blob/main/PLAYBOOK.md` },
	{ label: "Live contract", sub: "escrow package · cspr.live", href: "https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d" },
	{ label: "Roadmap", sub: "vision · what ships next", href: `${REPO}/blob/main/VISION.md` },
];

export function OneClickHub() {
	const [visible, setVisible] = useState(false);
	const [openHub, setOpenHub] = useState(false);
	// viewer preferences: pure-black background (produx mode) & art-free cards. Persisted.
	const [bgOn, setBgOn] = useState(true);
	const [artOn, setArtOn] = useState(true);

	useEffect(() => {
		const bg = localStorage.getItem("mesh-bg") !== "off";
		const art = localStorage.getItem("mesh-art") !== "off";
		setBgOn(bg);
		setArtOn(art);
		document.documentElement.classList.toggle("bg-off", !bg);
		document.documentElement.classList.toggle("art-off", !art);
	}, []);

	const toggleBg = () => setBgOn((v) => {
		const next = !v;
		localStorage.setItem("mesh-bg", next ? "on" : "off");
		document.documentElement.classList.toggle("bg-off", !next);
		return next;
	});
	const toggleArt = () => setArtOn((v) => {
		const next = !v;
		localStorage.setItem("mesh-art", next ? "on" : "off");
		document.documentElement.classList.toggle("art-off", !next);
		return next;
	});

	useEffect(() => {
		const onScroll = () => setVisible(window.scrollY > window.innerHeight * 0.8);
		window.addEventListener("scroll", onScroll, { passive: true });
		onScroll();
		return () => window.removeEventListener("scroll", onScroll);
	}, []);

	useEffect(() => {
		if (!openHub) return;
		const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setOpenHub(false); };
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [openHub]);

	const go = (row: (typeof ROWS)[number]) => {
		setOpenHub(false);
		if (row.href) window.open(row.href, "_blank", "noopener,noreferrer");
		else if (row.anchor) document.querySelector(row.anchor)?.scrollIntoView({ behavior: "smooth", block: "start" });
	};

	return (
		<>
			{/* capsule */}
			<AnimatePresence>
				{visible && !openHub && (
					<motion.button
						key="capsule"
						initial={{ opacity: 0, y: 26 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0, y: 26 }}
						transition={{ duration: 0.5, ease: glide }}
						onClick={() => setOpenHub(true)}
						className="group fixed bottom-[3.2vh] left-1/2 -translate-x-1/2 z-[8500] flex items-center justify-center gap-3.5 border border-white/[0.08] bg-[#0a0508]/45 backdrop-blur-md px-10 py-3.5 min-w-[240px] cursor-pointer transition-all duration-500 hover:border-[var(--red-700)] hover:bg-[#0a0508]/95"
						aria-label="Open the one-click hub"
					>
						<span className="size-[7px] bg-[var(--red-700)] opacity-40 transition-opacity duration-500 group-hover:opacity-100" style={{ animation: "crystalPulse 2.6s ease-in-out infinite" }} />
						<span className="label-13-mono uppercase tracking-[0.26em] text-white/55 transition-colors duration-500 group-hover:text-white">one click</span>
					</motion.button>
				)}
			</AnimatePresence>

			{/* panel */}
			<AnimatePresence>
				{openHub && (
					<>
						<motion.div
							key="hub-backdrop"
							className="fixed inset-0 z-[8550] bg-black/55 backdrop-blur-[3px]"
							initial={{ opacity: 0 }}
							animate={{ opacity: 1 }}
							exit={{ opacity: 0 }}
							transition={{ duration: 0.35 }}
							onClick={() => setOpenHub(false)}
						/>
						<motion.div
							key="hub-panel"
							initial={{ clipPath: "inset(100% 0% 0% 0%)", y: 24 }}
							animate={{ clipPath: "inset(0% 0% 0% 0%)", y: 0 }}
							exit={{ clipPath: "inset(100% 0% 0% 0%)", y: 24 }}
							transition={{ duration: 0.55, ease: glide }}
							className="fixed bottom-[3.2vh] left-1/2 -translate-x-1/2 z-[8600] w-[min(560px,92vw)] border border-white/15 bg-[#0a0508]/96 backdrop-blur-xl"
						>
							{/* header strip */}
							<div className="flex items-center justify-between border-b border-white/10 px-5 py-3.5">
								<span className="label-13-mono uppercase tracking-[0.22em] text-white/90">one click</span>
								<button onClick={() => setOpenHub(false)} className="label-13-mono text-white/60 hover:text-white transition-colors cursor-pointer" aria-label="Close hub">✕</button>
							</div>

							{/* rows */}
							<div className="flex flex-col">
								{ROWS.map((row, i) => (
									<motion.button
										key={row.label}
										initial={{ opacity: 0, y: 14, filter: "blur(5px)" }}
										animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
										transition={{ duration: 0.5, ease: glide, delay: 0.12 + i * 0.055 }}
										onClick={() => go(row)}
										className="group flex items-baseline justify-between gap-4 border-b border-white/[0.07] px-5 py-4 text-left cursor-pointer hover:bg-white/[0.04] transition-colors"
									>
										<span className="text-white/90 group-hover:text-white transition-colors" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(17px, 1.25vw, 24px)", fontWeight: 400 }}>
											{row.label}
										</span>
										<span className="label-12-mono text-white/35 group-hover:text-[var(--red-700)] transition-colors whitespace-nowrap">
											{row.sub} {row.href ? "↗" : "↓"}
										</span>
									</motion.button>
								))}
							</div>

							{/* the skill scroll — neco frame */}
							<motion.div
								initial={{ opacity: 0, y: 14, filter: "blur(5px)" }}
								animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
								transition={{ duration: 0.5, ease: glide, delay: 0.4 }}
								className="mx-5 my-4 border border-[var(--red-700)]/35 bg-[#120608]/80 px-4 py-3.5"
							>
								<p className="label-12-mono text-[var(--red-700)] tracking-[0.18em]">◢◤ casper agent skill ◥◣</p>
								<p className="label-12-mono text-white/55 mt-2 leading-[1.6] normal-case" style={{ textTransform: "none" }}>
									A machine-readable skill: any AI agent can discover the mesh, pay in CSPR and get
									judged — starting from <span className="text-white/80">GET /api/mcp</span>.
								</p>
								<div className="mt-3 flex gap-5">
									<a href={`${REPO}/blob/main/CASPER_AGENT_SKILL.md`} target="_blank" rel="noopener noreferrer" className="label-12-mono text-white/75 hover:text-[var(--red-700)] transition-colors">[ view skill ↗ ]</a>
									<a href={`${REPO}/raw/main/CASPER_AGENT_SKILL.md`} target="_blank" rel="noopener noreferrer" className="label-12-mono text-white/45 hover:text-white transition-colors">[ raw ↘ ]</a>
								</div>
							</motion.div>

							{/* viewer preferences — one-tap taste switches */}
						<motion.div
							initial={{ opacity: 0 }}
							animate={{ opacity: 1 }}
							transition={{ duration: 0.45, delay: 0.46 }}
							className="mx-5 mb-4 flex items-center justify-between gap-3"
						>
							<span className="label-12-mono text-white/35 tracking-[0.18em]">viewer/</span>
							<div className="flex gap-3">
								<button onClick={toggleBg} className="label-12-mono border border-white/12 px-3 py-2 text-white/70 hover:text-white hover:border-[var(--red-700)] transition-colors cursor-pointer">
									[ fabric · <span className={bgOn ? "text-[var(--red-700)]" : "text-white/35"}>{bgOn ? "on" : "off"}</span> ]
								</button>
								<button onClick={toggleArt} className="label-12-mono border border-white/12 px-3 py-2 text-white/70 hover:text-white hover:border-[var(--red-700)] transition-colors cursor-pointer">
									[ card art · <span className={artOn ? "text-[var(--red-700)]" : "text-white/35"}>{artOn ? "on" : "off"}</span> ]
								</button>
							</div>
						</motion.div>

						{/* bottom buttons */}
							<motion.div
								initial={{ opacity: 0 }}
								animate={{ opacity: 1 }}
								transition={{ duration: 0.45, delay: 0.5 }}
								className="grid grid-cols-2 gap-3 px-5 pb-5"
							>
								<a href={REPO} target="_blank" rel="noopener noreferrer" className="border border-white/15 py-3 text-center label-13-mono text-white/85 hover:border-[var(--red-700)] hover:text-white transition-colors">[ github ]</a>
								<a href="https://dorahacks.io/buidl/46714" target="_blank" rel="noopener noreferrer" className="border border-white/15 py-3 text-center label-13-mono text-white/85 hover:border-[var(--red-700)] hover:text-white transition-colors">[ dorahacks ]</a>
							</motion.div>
						</motion.div>
					</>
				)}
			</AnimatePresence>
		</>
	);
}
