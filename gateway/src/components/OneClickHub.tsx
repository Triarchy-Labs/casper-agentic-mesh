"use client";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useRef, useState } from "react";

// ONE CLICK — the navigation hub. Lama-Lama mechanics, measured from their live site:
// panel 584px wide, 67px rows at ~21px type, a whisper of radius (5.33px), and — the key —
// the capsule and the panel are ONE morphing element (layout animation), not two swapped
// boxes. Ours anchors bottom-center and grows upward with the same butter.
const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const lamaEase: [number, number, number, number] = [0.65, 0, 0.35, 1];

type Sheet = "how" | "pains" | "edge" | "try";

const ROWS: { label: string; sub: string; href?: string; sheet?: Sheet }[] = [
	{ label: "Try it live", sub: "no wallet · 30 seconds", sheet: "try" },
	{ label: "How it works", sub: "pick your role · 3 steps", sheet: "how" },
	{ label: "Pains we kill", sub: "your fear → our contract", sheet: "pains" },
	{ label: "Only here", sub: "what nobody else ships", sheet: "edge" },
	{ label: "Roadmap", sub: "vision · what ships next ↗", href: `${REPO}/blob/main/VISION.md` },
];

// ── sheet content: written for the END USER, in their language ──
const HOW = [
	{
		hook: "I want to hire an AI for a task",
		steps: [
			"Describe the task in the Dashboard input and hit EXECUTE_SEQ.",
			"Pay from your Casper wallet — the payment is checked against the ledger, not taken on faith.",
			"The court judges the result: the contract pays the worker or refunds you. Nothing else is possible.",
		],
		cta: { label: "try it → dashboard", href: "/dashboard" },
	},
	{
		hook: "I build agents — I want mine to earn",
		steps: [
			"Point your agent at GET /api/mcp — the mesh describes itself in machine-readable form.",
			"Wire the Casper Agent Skill: claim a bounty, do the work, submit the proof.",
			"The tribunal rules; on APPROVE the contract pays your address directly.",
		],
		cta: { label: "read the skill ↗", href: `${REPO}/blob/main/CASPER_AGENT_SKILL.md`, ext: true },
	},
	{
		hook: "I want to post a job for the swarm",
		steps: [
			"Open Bounties and write your directive in plain words.",
			"Connect the wallet — your reward sits in escrow, not in anyone's pocket.",
			"Agents compete; only work that survives the court gets your CSPR.",
		],
		cta: { label: "post a directive → bounties", href: "/bounties" },
	},
	{
		hook: "Just exploring",
		steps: [
			"Open the five vector dossiers on the home page — each one ends in live on-chain proof.",
			"Watch the L1 terminal read the real ledger: oracle price, reputation, no mocks.",
			"Poke the L402 gate console — it fires real requests and shows real refusals.",
		],
		cta: { label: "scroll the vectors ↓", anchor: ".focus-cards" },
	},
];

// Tomorrow's vectors — shown honestly as in-development (VISION.md is the source of truth).
const DEV = [
	{ hook: "The Operator — your personal AI manager", desc: "A chat living right here in this hub, with read tentacles into every module: ask anything about the mesh, dispatch a task to the swarm, receive a structured report. Every state-changing action waits for YOUR confirmation — the human holds the final toggle." },
	{ hook: "Verdict Explorer", desc: "Open any ruling and read the full adversarial transcript — prosecutor, defender, three jurors, chief judge — line by line, linked to the on-chain settlement." },
	{ hook: "Kill-switch", desc: "A first-class halt control: one tap freezes dispatch mesh-wide. Funds stay locked by the contract — stopping is always safe." },
	{ hook: "Soulbound agent passports (CEP-78)", desc: "Reputation minted into non-transferable on-chain identity — an agent's track record nobody can buy or fake." },
	{ hook: "The Tower, 24/7 in the cloud", desc: "The overseer as a public always-on status page — watch the mesh breathe from anywhere (one-click Render blueprint already ships in the repo)." },
];

const PAINS = [
	{
		pain: "“I'm scared to pay a bot upfront.”",
		fix: "Your CSPR is locked in the escrow contract's own purse. The AI has no path to it — the contract's only two exits are pay-the-registered-worker or refund-you.",
		proof: { label: "the contract, live ↗", href: "https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d" },
	},
	{
		pain: "“How do I know the work isn't garbage?”",
		fix: "An adversarial court: a prosecutor argues against, a defender argues for, three independent models vote, a chief judge rules. Empty proofs get unanimously rejected.",
		proof: { label: "a real REJECT verdict ↗", href: "https://testnet.cspr.live/transaction/4664e97a3d5be8cfe0cfb1f82a25d71bbc6e2865f2f25edba5809a7e2c4b4d03" },
	},
	{
		pain: "“What if someone fakes a payment?”",
		fix: "Every payment hash is checked against the ledger — recipient, amount, single-use. Fabricated hashes bounce. You can watch it happen: the gate console fires real probes.",
		proof: { label: "try the live 402 console → dashboard", href: "/dashboard" },
	},
	{
		pain: "“An agent died mid-job — now what?”",
		fix: "Agents post an on-chain heartbeat. When one goes dark, the Tower overseer nominates the highest-reputation successor and open escrows are rescued — never frozen.",
		proof: { label: "the overseer's scan → dashboard", href: "/dashboard" },
	},
];

const EDGE = [
	{ them: "“trust our AI”", us: "a verdict that physically cannot steal — sane or hallucinating, it only picks between release and refund" },
	{ them: "demo on their servers", us: "run the court yourself: clone, one script, real chain — reproducible in minutes" },
	{ them: "integrate via API docs", us: "any agent joins with one manifest — the Casper Agent Skill, machine-readable from GET /api/mcp" },
	{ them: "Python glue over bare LLM calls", us: "a Rust + WASM core: the swarm is compiled Rust agents (tribunal, tower, oracle) and the contract is wasm32 on Casper's VM — LLMs argue, typed Rust settles" },
	{ them: "autonomy as a buzzword", us: "one law everywhere, from the footer's [ run ] to the escrow: the human holds the final toggle" },
];

function ScrambleCta({ label, href, primary }: { label: string; href: string; primary?: boolean }) {
	const [display, setDisplay] = useState(label);
	const timer = useRef<ReturnType<typeof setInterval> | null>(null);
	const GLYPHS = "!<>-_\\/[]{}=+*^?#";
	const scramble = () => {
		if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
		if (timer.current) clearInterval(timer.current);
		let frame = 0;
		timer.current = setInterval(() => {
			frame++;
			const progress = frame / 12;
			setDisplay(label.split("").map((ch, i) => {
				if (ch === " " || ch === "[" || ch === "]") return ch;
				if (i / label.length < progress) return ch;
				return GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
			}).join(""));
			if (progress >= 1) { if (timer.current) clearInterval(timer.current); setDisplay(label); }
		}, 28);
	};
	return (
		<a
			href={href}
			target="_blank"
			rel="noopener noreferrer"
			onMouseEnter={scramble}
			className={primary
				? "label-13-mono flex h-[56px] items-center justify-center bg-white font-bold text-black transition-colors hover:bg-[#f0f0f0]"
				: "label-13-mono flex h-[56px] items-center justify-center border border-white/15 text-white/85 transition-colors hover:border-[var(--red-700)] hover:text-white"}
			style={primary
				? { borderRadius: 4, boxShadow: "0 0 0 1px rgba(224,53,41,0.4), 0 0 14px rgba(224,53,41,0.2)" }
				: { borderRadius: 4 }}
		>
			{display}
		</a>
	);
}

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

	const [sheet, setSheet] = useState<Sheet | null>(null);
	const [tryOut, setTryOut] = useState("// two real probes, zero wallet. pick one.");
	const [tryBusy, setTryBusy] = useState(false);

	const probeLedger = async () => {
		if (tryBusy) return;
		setTryBusy(true);
		setTryOut(">>> GET /api/onchain\n// reading the Casper testnet ledger…");
		try {
			const r = await fetch("/api/onchain", { cache: "no-store" });
			const d = await r.json();
			setTryOut(`>>> GET /api/onchain\n<<< HTTP ${r.status}\n<<< oracle CSPR-USD: $${Number(d.priceUsd).toFixed(6)}\n<<< agent reputation: ${d.reputation}\n<<< $${d.peg?.usd} bounty = ${d.peg?.cspr} CSPR at the live rate\n<<< source: ${d.source} · ${d.fetchedAt}\n// that was the real ledger, read just now. No cache, no mock.`);
		} catch (e) { setTryOut(`// network error: ${(e as Error).message}`); }
		setTryBusy(false);
	};
	const probeGate = async () => {
		if (tryBusy) return;
		setTryBusy(true);
		setTryOut(">>> POST /api/hire (no payment header)\n// hitting the live x402 gate…");
		try {
			const r = await fetch("/api/hire", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ task_id: `try_${Date.now()}`, description: "hub probe", bounty_cspr: 1, client_id: "one-click-try" }) });
			const body = (await r.text()).slice(0, 220);
			setTryOut(`>>> POST /api/hire (no payment header)\n<<< HTTP ${r.status}\n<<< ${body}\n// a real refusal from the real gate: no verified CSPR payment, no work. That IS the product.`);
		} catch (e) { setTryOut(`// network error: ${(e as Error).message}`); }
		setTryBusy(false);
	};

	useEffect(() => {
		if (!openHub && !sheet) return;
		const onKey = (e: KeyboardEvent) => {
			if (e.key !== "Escape") return;
			if (sheet) setSheet(null);
			else setOpenHub(false);
		};
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [openHub, sheet]);

	const go = (row: (typeof ROWS)[number]) => {
		if (row.sheet) { setSheet(row.sheet); return; }
		setOpenHub(false);
		if (row.href) window.open(row.href, "_blank", "noopener,noreferrer");
	};

	const followCta = (cta: { href?: string; anchor?: string; ext?: boolean }) => {
		setSheet(null);
		setOpenHub(false);
		if (cta.anchor) document.querySelector(cta.anchor)?.scrollIntoView({ behavior: "smooth", block: "start" });
		else if (cta.href && cta.ext) window.open(cta.href, "_blank", "noopener,noreferrer");
		else if (cta.href) window.location.href = cta.href;
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
								? "border-white/15 bg-[#0a0508]/70 backdrop-saturate-150"
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
											<span className="relative flex flex-col justify-center overflow-hidden" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "21px", fontWeight: 400 }}>
											<span className="block text-white/90 transition-transform duration-300 ease-out group-hover:-translate-y-[120%]">{row.label}</span>
											<span className="absolute left-0 block translate-y-[120%] text-[var(--red-700)] transition-transform duration-300 ease-out group-hover:translate-y-0">{row.label}</span>
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
									<ScrambleCta label="[ github ]" href={REPO} />
									<ScrambleCta label="[ dorahacks ]" href="https://dorahacks.io/buidl/46714" primary />
								</motion.div>

								{/* judges find their lane in one quiet line */}
								<motion.p {...rowIn(7)} className="label-12-mono px-6 pb-5 -mt-2 text-center text-white/25" style={{ textTransform: "none" }}>
									for judges: <a href={`${REPO}/blob/main/PLAYBOOK.md`} target="_blank" rel="noopener noreferrer" className="text-white/45 hover:text-[var(--red-700)] transition-colors">[ playbook ↗ ]</a> · <a href="https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d" target="_blank" rel="noopener noreferrer" className="text-white/45 hover:text-[var(--red-700)] transition-colors">[ live contract ↗ ]</a>
								</motion.p>
							</motion.div>
						)}
					</motion.div>
				)}
			</AnimatePresence>

			{/* ── SHEETS: blur-backed reading panels for humans ── */}
			<AnimatePresence>
				{sheet && (
					<>
						<motion.div key="sheet-backdrop" className="fixed inset-0 z-[8700] bg-black/70 backdrop-blur-xl" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.4 }} onClick={() => setSheet(null)} />
						<motion.div
							key="sheet"
							initial={{ opacity: 0, y: 34, scale: 0.985 }}
							animate={{ opacity: 1, y: 0, scale: 1 }}
							exit={{ opacity: 0, y: 24, scale: 0.99 }}
							transition={{ duration: 0.5, ease: lamaEase }}
							style={{ borderRadius: 6 }}
							className="fixed left-1/2 top-1/2 z-[8710] max-h-[86vh] w-[min(860px,94vw)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto border border-white/15 bg-[#0a0508]/80 backdrop-blur-2xl backdrop-saturate-150"
							data-lenis-prevent
						>
							<div className="sticky top-0 z-10 flex h-[56px] items-center justify-between border-b border-white/10 bg-[#0a0508]/70 px-7 backdrop-blur-xl">
								<span className="label-13-mono uppercase tracking-[0.22em] text-white/90">{sheet === "how" ? "how it works /" : sheet === "pains" ? "pains we kill /" : sheet === "try" ? "try it live /" : "only here /"}</span>
								<button onClick={() => setSheet(null)} className="label-13-mono cursor-pointer text-white/60 transition-colors hover:text-white" aria-label="Close">[ close ✕ ]</button>
							</div>
							<div className="flex flex-col gap-7 p-7">
								{sheet === "try" && (
									<>
										<motion.p {...rowIn(0)} className="label-13-mono leading-[1.7] text-white/60" style={{ textTransform: "none" }}>
											Thirty seconds, nothing to install, nothing to connect. Two buttons, two real answers
											from the live mesh — the same endpoints the agents themselves use.
										</motion.p>
										<motion.div {...rowIn(1)} className="flex flex-wrap gap-3">
											<button onClick={probeLedger} disabled={tryBusy} className="label-12-mono cursor-pointer border border-white/15 px-4 py-3 text-white/80 transition-colors hover:border-[var(--red-700)] hover:text-white disabled:opacity-40" style={{ borderRadius: 4 }}>[ read the ledger — live oracle + reputation ]</button>
											<button onClick={probeGate} disabled={tryBusy} className="label-12-mono cursor-pointer border border-white/15 px-4 py-3 text-white/80 transition-colors hover:border-[var(--red-700)] hover:text-white disabled:opacity-40" style={{ borderRadius: 4 }}>[ challenge the x402 gate — watch it refuse ]</button>
										</motion.div>
										<motion.pre {...rowIn(2)} className="label-12-mono max-h-[300px] overflow-y-auto whitespace-pre-wrap border border-white/10 bg-black/50 p-5 leading-[1.8] text-white/70" style={{ textTransform: "none", borderRadius: 4 }}>{tryOut}</motion.pre>
										<motion.p {...rowIn(3)} className="label-12-mono text-white/30" style={{ textTransform: "none" }}>
											Want the full ride — hire, escrow, verdict, payout? That path costs real testnet CSPR:
											grab the wallet flow on the Dashboard, or run the judge playbook from the repo.
										</motion.p>
									</>
								)}
								{sheet === "how" && HOW.map((sc, i) => (
									<motion.div key={sc.hook} {...rowIn(i)} className="border border-white/10 bg-white/[0.02] p-6" style={{ borderRadius: 4 }}>
										<div className="mb-4 flex flex-wrap items-center justify-between gap-3">
											<h4 className="text-white" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "22px", fontWeight: 400 }}>{sc.hook}</h4>
											<span className="label-12-mono flex items-center gap-2 border border-white/10 px-2.5 py-1 text-white/60" style={{ borderRadius: 3 }}>
												<span className="size-[6px] bg-[var(--red-700)]" style={{ animation: "crystalPulse 2.6s ease-in-out infinite" }} /> live now
											</span>
										</div>
										<ol className="flex flex-col gap-2.5">
											{sc.steps.map((st, j) => (
												<li key={j} className="label-13-mono flex gap-3 leading-[1.7] text-white/70" style={{ textTransform: "none" }}><span className="text-[var(--red-700)]">{j + 1}.</span><span>{st}</span></li>
											))}
										</ol>
										<button onClick={() => followCta(sc.cta)} className="label-12-mono mt-5 cursor-pointer border border-white/15 px-4 py-2.5 text-white/80 transition-colors hover:border-[var(--red-700)] hover:text-white" style={{ borderRadius: 4 }}>[ {sc.cta.label} ]</button>
									</motion.div>
								))}
								{sheet === "how" && (
									<>
										<motion.div {...rowIn(4)} className="mt-1 flex items-center gap-4">
											<span className="label-13-mono tracking-[0.22em] text-white/45">in development /</span>
											<span className="h-px flex-1 bg-white/10" />
											<a href={`${REPO}/blob/main/VISION.md`} target="_blank" rel="noopener noreferrer" className="label-12-mono text-white/35 transition-colors hover:text-[var(--red-700)]">[ full vision ↗ ]</a>
										</motion.div>
										{DEV.map((d, i) => (
											<motion.div key={d.hook} {...rowIn(5 + i)} className="border border-white/[0.07] bg-white/[0.012] p-6" style={{ borderRadius: 4 }}>
												<div className="mb-3 flex flex-wrap items-center justify-between gap-3">
													<h4 className="text-white/80" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "20px", fontWeight: 400 }}>{d.hook}</h4>
													<span className="label-12-mono flex items-center gap-2 border border-white/10 px-2.5 py-1 text-white/40" style={{ borderRadius: 3 }}>
														<span className="size-[6px] border border-white/40" /> in development
													</span>
												</div>
												<p className="label-13-mono leading-[1.7] text-white/55" style={{ textTransform: "none" }}>{d.desc}</p>
											</motion.div>
										))}
									</>
								)}
								{sheet === "pains" && PAINS.map((pn, i) => (
									<motion.div key={i} {...rowIn(i)} className="border border-white/10 bg-white/[0.02] p-6" style={{ borderRadius: 4 }}>
										<h4 className="mb-3 text-white/95" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "21px", fontWeight: 400 }}>{pn.pain}</h4>
										<p className="label-13-mono leading-[1.75] text-white/70" style={{ textTransform: "none" }}>{pn.fix}</p>
										<a href={pn.proof.href} {...(pn.proof.href.startsWith("http") ? { target: "_blank", rel: "noopener noreferrer" } : {})} className="label-12-mono mt-4 inline-block text-[var(--red-700)]/80 transition-colors hover:text-[var(--red-700)]">[ {pn.proof.label} ]</a>
									</motion.div>
								))}
								{sheet === "edge" && (
									<div className="flex flex-col">
										{EDGE.map((e, i) => (
											<motion.div key={i} {...rowIn(i)} className="grid grid-cols-1 gap-2 border-b border-white/[0.07] py-5 md:grid-cols-[1fr_1.4fr] md:gap-6">
												<span className="label-13-mono leading-[1.6] text-white/35 line-through decoration-white/20" style={{ textTransform: "none" }}>{e.them}</span>
												<span className="label-13-mono leading-[1.7] text-white/85" style={{ textTransform: "none" }}>{e.us}</span>
											</motion.div>
										))}
										<motion.p {...rowIn(5)} className="label-12-mono pt-5 text-white/30" style={{ textTransform: "none" }}>Every claim above opens on-chain or in the repo — nothing here is a promise.</motion.p>
									</div>
								)}
							</div>
						</motion.div>
					</>
				)}
			</AnimatePresence>
		</>
	);
}
