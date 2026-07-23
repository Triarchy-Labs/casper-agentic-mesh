"use client";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect } from "react";

// produx project-detail mechanic, our substance. A card's art expands into a full-screen dossier
// hero (clip-wipe, their natureSway ease), then their Art Direction section rhythm carries our
// verifiable content: what it is / how it works / how it synergizes / verify on-chain. Media frames
// are deliberate placeholders (user fills bespoke art later). Close via ×, ESC, or backdrop.

const EX = "https://testnet.cspr.live";
const REPO = "https://github.com/Triarchy-Labs/casper-agentic-mesh";
const nature: [number, number, number, number] = [0.08, 0.494, 0.14, 1];
const glide: [number, number, number, number] = [0.1, 0.4, 0.15, 1];

type Proof = { label: string; href: string };
type Vector = {
	slug: string;
	index: string; // 01..05
	kicker: string;
	title: string;
	art: string;
	tags: string[];
	lede: string; // one-line what-it-is
	how: string[]; // mechanism, step lines
	synergy: string; // how it plugs into the other vectors
	proof: Proof[];
	frames: number; // placeholder media frames
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
	},
	{
		slug: "oracle",
		index: "03",
		kicker: "vector beta",
		title: "RWA Risk Oracle",
		art: "/vector_oracle.jpeg",
		tags: ["Oracle", "Identity", "Reputation"],
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
	},
	{
		slug: "tribunal",
		index: "04",
		kicker: "vector gamma",
		title: "Agent Tribunal",
		art: "/card-tribunal.jpg",
		tags: ["Adversarial", "5 LLMs", "On-chain"],
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
	},
	{
		slug: "x402",
		index: "05",
		kicker: "vector delta",
		title: "x402 Payment Layer",
		art: "/card-x402.jpg",
		tags: ["HTTP 402", "Pay-per-call", "CSPR"],
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
	},
];

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

export function VectorDossier({ slug, onClose }: { slug: string | null; onClose: () => void }) {
	const v = VECTORS.find((x) => x.slug === slug) || null;

	useEffect(() => {
		if (!v) return;
		const prev = document.body.style.overflow;
		document.body.style.overflow = "hidden";
		const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
		window.addEventListener("keydown", onKey);
		return () => { document.body.style.overflow = prev; window.removeEventListener("keydown", onKey); };
	}, [v, onClose]);

	return (
		<AnimatePresence>
			{v && (
				<motion.div
					key={v.slug}
					className="fixed inset-0 z-[9999] overflow-y-auto overflow-x-hidden"
					style={{ backgroundColor: "#050305" }}
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					exit={{ opacity: 0 }}
					transition={{ duration: 0.4, ease: glide }}
				>
					{/* close */}
					<button
						onClick={onClose}
						aria-label="Close"
						className="group fixed top-[3vh] right-[3vw] z-[10000] flex items-center gap-2 label-13-mono text-white/70 hover:text-white transition-colors"
					>
						<span className="tracking-[0.16em]">[ CLOSE</span>
						<span className="inline-block transition-transform duration-300 group-hover:rotate-90">✕</span>
						<span className="tracking-[0.16em]">]</span>
					</button>

					<div className="mx-auto w-full max-w-[1500px] px-[5.5vw] pb-[16vh]">
						{/* HERO — the card art expands to the top (clip-wipe, their mechanic) */}
						<div className="pt-[12vh]">
							<motion.div
								initial={{ clipPath: "inset(0% 0% 100% 0%)", scale: 1.06 }}
								animate={{ clipPath: "inset(0% 0% 0% 0%)", scale: 1 }}
								transition={{ duration: 1, ease: nature }}
								className="relative w-full aspect-[2.2/1] max-sm:aspect-[1.4/1] overflow-hidden bg-black"
							>
								<div
									className="absolute inset-0 bg-cover bg-center"
									style={{ backgroundImage: `url(${v.art})`, transform: "scale(1.04)" }}
								/>
								<div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(to top, rgba(5,3,5,0.85), transparent 55%)" }} />
							</motion.div>
						</div>

						{/* title block */}
						<motion.div
							initial={{ opacity: 0, y: 24 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 0.9, ease: glide, delay: 0.15 }}
							className="mt-[5vh] flex flex-wrap items-end justify-between gap-6"
						>
							<div>
								<span className="label-14-mono text-[var(--red-700)]">{v.index} // {v.kicker}</span>
								<h2 className="mt-3 uppercase leading-[0.95]" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontSize: "clamp(40px, 6vw, 104px)", fontWeight: 400 }}>
									{v.title}
								</h2>
							</div>
							<div className="flex gap-2 flex-wrap pb-2">
								{v.tags.map((t) => <span key={t} className="nb-tag">{t}</span>)}
							</div>
						</motion.div>

						<motion.p
							initial={{ opacity: 0, y: 20 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 0.9, ease: glide, delay: 0.25 }}
							className="mt-[4vh] max-w-[62ch] text-white/80"
							style={{ fontFamily: "var(--font-DM-mono), var(--font-mono), monospace", fontSize: "clamp(16px, 1.35vw, 26px)", lineHeight: 1.5 }}
						>
							{v.lede}
						</motion.p>

						{/* SECTIONS — produx Art Direction rhythm */}
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

							{/* placeholder media band — user drops bespoke art here later */}
							<div className="grid grid-cols-12 gap-[1.39vw]">
								{Array.from({ length: v.frames }).map((_, i) => (
									<div
										key={i}
										className={`relative overflow-hidden border border-white/10 bg-white/[0.02] ${v.frames === 2 ? "col-span-12 md:col-span-6 aspect-[1.4/1]" : i === 0 ? "col-span-12 aspect-[2.6/1]" : "col-span-12 md:col-span-6 aspect-[1.6/1]"}`}
									>
										<div className="absolute inset-0 flex items-center justify-center">
											<span className="label-12-mono text-white/25 tracking-[0.2em]">◢ MEDIA {v.index}.{i + 1} ◣</span>
										</div>
										<div className="absolute left-3 top-3 label-12-mono text-white/20">{v.slug}-img-{i + 1}</div>
									</div>
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
										<a
											key={p.href}
											href={p.href}
											target="_blank"
											rel="noopener noreferrer"
											className="group label-13-mono uppercase tracking-[0.12em] text-white/70 hover:text-[var(--red-700)] transition-colors"
										>
											[ {p.label} ]
										</a>
									))}
								</div>
								<p className="mt-6 label-12-mono text-white/35">Don't trust this panel — open the hashes. Full path: PLAYBOOK.md</p>
							</Section>
						</div>
					</div>
				</motion.div>
			)}
		</AnimatePresence>
	);
}
