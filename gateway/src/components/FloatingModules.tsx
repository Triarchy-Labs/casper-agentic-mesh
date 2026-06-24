"use client";
import React, { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";

const FONT_HEADING = "'Helvetica Now Display', 'Inter', sans-serif";
const FONT_MONO = "'SF Mono', 'Fira Code', monospace";

interface ModuleInfo {
	id: string;
	name: string;
	description: string;
	position: { top?: string; bottom?: string; left?: string; right?: string };
}

const MODULES: ModuleInfo[] = [
	{ id: "wasm", name: "WASM SANDBOX", description: "Isolated execution environment for untrusted agent code", position: { top: "10%", left: "4%" } },
	{ id: "dex", name: "DEX ROUTER", description: "Cross-chain liquidity aggregation layer", position: { top: "8%", right: "4%" } },
	{ id: "security", name: "SECURITY MATRIX", description: "Zero-trust verification mesh for all transactions", position: { top: "42%", left: "2%" } },
	{ id: "risk", name: "RISK ENGINE", description: "Real-time position & exposure analysis", position: { top: "40%", right: "2%" } },
	{ id: "consensus", name: "CONSENSUS BRIDGE", description: "Multi-validator signature coordination protocol", position: { bottom: "18%", right: "5%" } },
	{ id: "telemetry", name: "TELEMETRY CORE", description: "Network health & latency monitoring daemon", position: { bottom: "15%", left: "5%" } },
];

// ===== TIMING CONSTANTS (calculated for comfortable reading) =====
const CYCLE_INTERVAL = 3000;   // New module every 3s
const MODULE_LIFETIME = 9000;  // Each module visible for 9s (3 cycles)
const ALL_REVEAL_DELAY = 30000; // 30s hold → all 6 appear (rare event)
const TYPEWRITER_SPEED = 50;   // 50ms per character (~600ms for 12-char name)

function TypewriterText({ text, speed = TYPEWRITER_SPEED }: { text: string; speed?: number }) {
	const [displayed, setDisplayed] = useState("");
	const [done, setDone] = useState(false);
	useEffect(() => {
		setDisplayed("");
		setDone(false);
		let i = 0;
		const interval = setInterval(() => {
			if (i < text.length) {
				setDisplayed(text.slice(0, i + 1));
				i++;
			} else {
				setDone(true);
				clearInterval(interval);
			}
		}, speed);
		return () => clearInterval(interval);
	}, [text, speed]);
	return <>{displayed}{!done && <span style={{ opacity: 0.4, animation: "blink 1s steps(2) infinite" }}>▌</span>}</>;
}

function ModuleCard({ module, showSnake }: { module: ModuleInfo; showSnake: boolean }) {
	return (
		<motion.div
			layout
			initial={{ opacity: 0, scale: 0.85, filter: "blur(8px)" }}
			animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
			exit={{ opacity: 0, scale: 0.9, filter: "blur(6px)" }}
			transition={{ duration: 0.8, ease: [0.4, 0, 0.2, 1] }}
			className={showSnake ? "snake-border" : ""}
			style={{
				position: "absolute",
				...module.position,
				width: 210,
				padding: "14px 16px",
				background: "rgba(255,255,255,0.05)",
				backdropFilter: "blur(24px) saturate(1.2)",
				WebkitBackdropFilter: "blur(24px) saturate(1.2)",
				border: showSnake ? "1px solid rgba(0,255,65,0.2)" : "1px solid rgba(255,255,255,0.08)",
				borderRadius: 12,
				zIndex: 30,
				pointerEvents: "none",
			}}
		>
			<div style={{
				fontFamily: FONT_HEADING,
				fontSize: "0.65rem",
				fontWeight: 600,
				letterSpacing: "0.14em",
				color: showSnake ? "#00ff41" : "rgba(255,255,255,0.8)",
				marginBottom: 6,
				transition: "color 1s ease",
			}}>
				<TypewriterText text={module.name} />
			</div>
			<div style={{
				fontFamily: FONT_MONO,
				fontSize: "0.55rem",
				color: "rgba(255,255,255,0.4)",
				lineHeight: 1.6,
				letterSpacing: "0.02em",
			}}>
				{module.description}
			</div>
			{/* Subtle glow indicator dot */}
			<div style={{
				position: "absolute",
				top: 8,
				right: 10,
				width: 4,
				height: 4,
				borderRadius: "50%",
				background: showSnake ? "#00ff41" : "rgba(255,255,255,0.3)",
				boxShadow: showSnake ? "0 0 8px rgba(0,255,65,0.6)" : "none",
				transition: "all 1s",
			}} />
		</motion.div>
	);
}

export default function FloatingModules({ hovered }: { hovered: boolean }) {
	// Each entry: { id, appearedAt }
	const [activeModules, setActiveModules] = useState<{ id: string; appearedAt: number }[]>([]);
	const [allMode, setAllMode] = useState(false);
	const cycleRef = useRef<ReturnType<typeof setInterval> | null>(null);
	const allTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const lifetimeRef = useRef<ReturnType<typeof setInterval> | null>(null);
	const reverseTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const reverseIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
	const nextIndexRef = useRef(0);

	useEffect(() => {
		if (!hovered) {
			// Mouse left: graceful fade out (AnimatePresence handles exit)
			// eslint-disable-next-line react-hooks/set-state-in-effect
			setActiveModules([]);
			setAllMode(false);
			nextIndexRef.current = 0;
			if (cycleRef.current) clearInterval(cycleRef.current);
			if (allTimerRef.current) clearTimeout(allTimerRef.current);
			if (lifetimeRef.current) clearInterval(lifetimeRef.current);
			if (reverseTimeoutRef.current) clearTimeout(reverseTimeoutRef.current);
			if (reverseIntervalRef.current) clearInterval(reverseIntervalRef.current);
			return;
		}

		// ── Breathing cycle: add one module every CYCLE_INTERVAL ──
		// Show first one immediately
		const firstMod = MODULES[0];
		setActiveModules([{ id: firstMod.id, appearedAt: Date.now() }]);
		nextIndexRef.current = 1;

		cycleRef.current = setInterval(() => {
			const now = Date.now();
			const nextMod = MODULES[nextIndexRef.current % MODULES.length];
			nextIndexRef.current++;

			setActiveModules(prev => {
				// Add new module
				const next = [...prev.filter(m => m.id !== nextMod.id), { id: nextMod.id, appearedAt: now }];
				// Remove expired (older than MODULE_LIFETIME)
				return next.filter(m => now - m.appearedAt < MODULE_LIFETIME);
			});
		}, CYCLE_INTERVAL);

		// ── Lifetime cleanup ticker (every 1s check for expired) ──
		lifetimeRef.current = setInterval(() => {
			const now = Date.now();
			setActiveModules(prev => prev.filter(m => now - m.appearedAt < MODULE_LIFETIME));
		}, 1000);

		// ── Rare event: 30s sustained hover → reveal ALL + snake ──
		allTimerRef.current = setTimeout(() => {
			setAllMode(true);
			setActiveModules(MODULES.map(m => ({ id: m.id, appearedAt: Date.now() })));
			// Stop cycling — all are now permanently visible
			if (cycleRef.current) clearInterval(cycleRef.current);
			if (lifetimeRef.current) clearInterval(lifetimeRef.current);

			// After another 30 seconds, fade them out in reverse order
			reverseTimeoutRef.current = setTimeout(() => {
				let reverseIndex = MODULES.length - 1;
				reverseIntervalRef.current = setInterval(() => {
					if (reverseIndex >= 0) {
						const targetId = MODULES[reverseIndex].id;
						setActiveModules(prev => prev.filter(m => m.id !== targetId));
						reverseIndex--;
					} else {
						if (reverseIntervalRef.current) clearInterval(reverseIntervalRef.current);
						setAllMode(false);
					}
				}, 600); // Pop one off every 600ms
			}, 30000);

		}, ALL_REVEAL_DELAY);

		return () => {
			if (cycleRef.current) clearInterval(cycleRef.current);
			if (allTimerRef.current) clearTimeout(allTimerRef.current);
			if (lifetimeRef.current) clearInterval(lifetimeRef.current);
			if (reverseTimeoutRef.current) clearTimeout(reverseTimeoutRef.current);
			if (reverseIntervalRef.current) clearInterval(reverseIntervalRef.current);
		};
	}, [hovered]);

	const visibleModules = MODULES.filter(m => activeModules.some(a => a.id === m.id));

	return (
		<AnimatePresence mode="popLayout">
			{visibleModules.map(mod => (
				<ModuleCard key={mod.id} module={mod} showSnake={allMode} />
			))}
		</AnimatePresence>
	);
}
