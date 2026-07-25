"use client";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";

// ONE CLICK — the navigation hub. Lama-Lama mechanics, measured from their live site:
// panel 584px wide, 67px rows at ~21px type, a whisper of radius (5.33px), and — the key —
// the capsule and the panel are ONE morphing element (layout animation), not two swapped
// boxes. Ours anchors bottom-center and grows upward with the same butter.
const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const lamaEase: [number, number, number, number] = [0.65, 0, 0.35, 1];

const ROWS: { label: string; sub: string; href?: string; anchor?: string }[] = [
	{ label: "How it works", sub: "five vectors · the cards ↓", anchor: ".focus-cards" },
	{ label: "Judge playbook", sub: "verify everything · 10 min ↗", href: `${REPO}/blob/main/PLAYBOOK.md` },
	{ label: "Live contract", sub: "escrow package · cspr.live ↗", href: "https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d" },
	{ label: "Roadmap", sub: "vision · what ships next ↗", href: `${REPO}/blob/main/VISION.md` },
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

	const rowIn = (i: number) => ({
		initial: { opacity: 0, y: 14, filter: "blur(5px)" },
		animate: { opacity: 1, y: 0, filter: "blur(0px)" },
		transition: { duration: 0.5, ease: lamaEase, delay: 0.16 + i * 0.05 },
	});

	return (
		<>
			{/* backdrop */}
			<AnimatePresence>
				{openHub && (
					<motion.div
						key="hub-backdrop"
						className="fixed inset-0 z-[8550] bg-black/55 backdrop-blur-[3px]"
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						transition={{ duration: 0.35 }}
						onClick={() => setOpenHub(false)}
					/>
				)}
			</AnimatePresence>

			{/* THE morphing element: capsule <-> panel, one box, lama-style */}
			<AnimatePresence>
				{visible && (
					<motion.div
						key="hub"
						layout
						initial={{ opacity: 0, y: 26 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0, y: 26 }}
						transition={{ layout: { duration: 0.55, ease: lamaEase }, duration: 0.45, ease: lamaEase }}
						style={{ borderRadius: 6 }}
						className={`fixed bottom-[3.2vh] left-1/2 w-[min(584px,92vw)] -translate-x-1/2 z-[8600] overflow-hidden border backdrop-blur-xl ${
							openHub
								? "border-white/15 bg-[#0a0508]/96"
								: "border-white/[0.08] bg-[#0a0508]/45"
						}`}
					>
						{!openHub ? (
							/* ── capsule state ── */
							<motion.button
								key="capsule"
								layout="position"
								onClick={() => setOpenHub(true)}
								className="group flex h-[64px] w-full cursor-pointer items-center justify-between px-6 transition-colors duration-500 hover:bg-[#0a0508]/60"
								aria-label="Open the one-click hub"
							>
								<span className="flex items-center gap-3.5">
									<span className="size-[7px] bg-[var(--red-700)] opacity-40 transition-opacity duration-500 group-hover:opacity-100" style={{ animation: "crystalPulse 2.6s ease-in-out infinite" }} />
									<span className="label-13-mono uppercase tracking-[0.26em] text-white/70 transition-colors duration-500 group-hover:text-white">one click</span>
								</span>
								<span className="label-12-mono hidden tracking-[0.18em] text-white/25 transition-colors duration-500 group-hover:text-white/45 sm:block">your mesh · one touch</span>
								<span className="text-[18px] leading-none text-white/50 transition-colors duration-500 group-hover:text-white">≡</span>
							</motion.button>
						) : (
							/* ── panel state ── */
							<motion.div key="panel" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.3, delay: 0.12 }}>
								{/* header strip */}
								<div className="flex h-[56px] items-center justify-between border-b border-white/10 px-6">
									<span className="flex items-center gap-3">
										<span className="size-[7px] bg-[var(--red-700)]" style={{ animation: "crystalPulse 2.6s ease-in-out infinite" }} />
										<span className="label-13-mono uppercase tracking-[0.22em] text-white/90">one click</span>
									</span>
									<button onClick={() => setOpenHub(false)} className="label-13-mono cursor-pointer text-white/60 transition-colors hover:text-white" aria-label="Close hub">—</button>
								</div>

								{/* rows — lama metrics: 64px tall, ~21px type */}
								<div className="flex flex-col">
									{ROWS.map((row, i) => (
										<motion.button
											key={row.label}
											{...rowIn(i)}
											onClick={() => go(row)}
											className="group flex min-h-[64px] cursor-pointer items-center justify-between gap-4 border-b border-white/[0.07] px-6 text-left transition-colors hover:bg-white/[0.04]"
										>
											<span className="text-white/90 transition-colors group-hover:text-white" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "21px", fontWeight: 400 }}>
												{row.label}
											</span>
											<span className="label-12-mono whitespace-nowrap text-white/35 transition-colors group-hover:text-[var(--red-700)]">
												{row.sub}
											</span>
										</motion.button>
									))}
								</div>

								{/* the skill scroll — neco frame (their 'OUR PITCHDECK' ghost slot) */}
								<motion.div {...rowIn(4)} className="mx-6 my-5 border border-[var(--red-700)]/35 bg-[#120608]/80 px-4 py-3.5" style={{ borderRadius: 4 }}>
									<p className="label-12-mono text-[var(--red-700)] tracking-[0.18em]">◢◤ casper agent skill ◥◣</p>
									<p className="label-12-mono mt-2 leading-[1.6] text-white/55" style={{ textTransform: "none" }}>
										A machine-readable skill: any AI agent can discover the mesh, pay in CSPR and get
										judged — starting from <span className="text-white/80">GET /api/mcp</span>.
									</p>
									<div className="mt-3 flex gap-5">
										<a href={`${REPO}/blob/main/CASPER_AGENT_SKILL.md`} target="_blank" rel="noopener noreferrer" className="label-12-mono text-white/75 transition-colors hover:text-[var(--red-700)]">[ view skill ↗ ]</a>
										<a href={`${REPO}/raw/main/CASPER_AGENT_SKILL.md`} target="_blank" rel="noopener noreferrer" className="label-12-mono text-white/45 transition-colors hover:text-white">[ raw ↘ ]</a>
									</div>
								</motion.div>

								{/* viewer preferences */}
								<motion.div {...rowIn(5)} className="mx-6 mb-5 flex items-center justify-between gap-3">
									<span className="label-12-mono tracking-[0.18em] text-white/35">viewer/</span>
									<div className="flex gap-3">
										<button onClick={toggleBg} className="label-12-mono cursor-pointer border border-white/15 px-3 py-2 text-white/70 transition-colors hover:border-[var(--red-700)] hover:text-white" style={{ borderRadius: 4 }}>
											[ fabric · <span className={bgOn ? "text-[var(--red-700)]" : "text-white/35"}>{bgOn ? "on" : "off"}</span> ]
										</button>
										<button onClick={toggleArt} className="label-12-mono cursor-pointer border border-white/15 px-3 py-2 text-white/70 transition-colors hover:border-[var(--red-700)] hover:text-white" style={{ borderRadius: 4 }}>
											[ card art · <span className={artOn ? "text-[var(--red-700)]" : "text-white/35"}>{artOn ? "on" : "off"}</span> ]
										</button>
									</div>
								</motion.div>

								{/* bottom CTA pair — lama's light-primary move */}
								<motion.div {...rowIn(6)} className="grid grid-cols-2 gap-3 px-6 pb-6">
									<a href={REPO} target="_blank" rel="noopener noreferrer" className="label-13-mono flex h-[56px] items-center justify-center border border-white/15 text-white/85 transition-colors hover:border-[var(--red-700)] hover:text-white" style={{ borderRadius: 4 }}>[ github ]</a>
									<a href="https://dorahacks.io/buidl/46714" target="_blank" rel="noopener noreferrer" className="label-13-mono flex h-[56px] items-center justify-center bg-white font-bold text-black transition-colors hover:bg-[#f0f0f0]" style={{ borderRadius: 4, boxShadow: "0 0 0 1px rgba(224,53,41,0.4), 0 0 14px rgba(224,53,41,0.2)" }}>[ dorahacks ]</a>
								</motion.div>
							</motion.div>
						)}
					</motion.div>
				)}
			</AnimatePresence>
		</>
	);
}
