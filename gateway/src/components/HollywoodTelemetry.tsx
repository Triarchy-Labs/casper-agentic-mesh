"use client";
import React, { useEffect, useRef, useState } from "react";

// LIVE Casper feed via the official CSPR.cloud API (Buildathon AI Toolkit). Every line is a
// REAL block or deploy pulled from testnet through our server-side /api/casper-stream (the key
// stays hidden). No random telemetry — if the feed can't reach cspr.cloud it says so, honestly.
export default function HollywoodTelemetry() {
	const [logs, setLogs] = useState<{ id: string; text: string; kind: "block" | "deploy" | "sys" }[]>([
		{ id: "boot", text: "[SYS] opening cspr.cloud stream · testnet…", kind: "sys" },
	]);
	const [live, setLive] = useState(false);
	const [height, setHeight] = useState<number | null>(null);
	const seen = useRef<Set<string>>(new Set());
	const boxRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		let stop = false;
		const pull = async () => {
			try {
				const r = await fetch("/api/casper-stream", { cache: "no-store" });
				const d = await r.json();
				if (stop) return;
				if (!d.ok || !d.lines?.length) {
					setLive(false);
					setLogs((prev) => [...prev.slice(-40), { id: `deg-${Date.now()}`, text: `[SYS] ${d.error || "feed frozen — no data faked"}`, kind: "sys" }]);
					return;
				}
				setLive(true);
				if (d.height) setHeight(d.height);
				const fresh = (d.lines as string[])
					.filter((l) => !seen.current.has(l))
					.map((l) => {
						seen.current.add(l);
						return { id: l, text: l, kind: (l.startsWith("[DEPLOY") ? "deploy" : "block") as "block" | "deploy" };
					});
				if (fresh.length) setLogs((prev) => [...prev, ...fresh].slice(-48));
			} catch {
				if (!stop) setLive(false);
			}
		};
		pull();
		const int = setInterval(pull, 4000);
		return () => { stop = true; clearInterval(int); };
	}, []);

	useEffect(() => { if (boxRef.current) boxRef.current.scrollTop = boxRef.current.scrollHeight; }, [logs]);

	return (
		<div className="relative flex h-full flex-col overflow-hidden">
			<div className="flex items-center justify-between border-b border-white/10 px-1 pb-3">
				<span className="label-13-mono tracking-[0.2em] text-white/60">CASPER LIVE STREAM</span>
				<span className="label-12-mono flex items-center gap-2 text-white/45">
					<span className={`size-[6px] rounded-full ${live ? "bg-[var(--red-700)]" : "bg-white/30"}`} style={live ? { animation: "crystalPulse 2s ease-in-out infinite" } : undefined} />
					cspr.cloud{height ? ` · #${height}` : ""}{live ? " · live" : " · …"}
				</span>
			</div>
			<div ref={boxRef} className="mt-3 flex flex-1 flex-col gap-1.5 overflow-y-auto pr-1" data-lenis-prevent>
				{logs.map((l) => (
					<div key={l.id} className={`label-12-mono leading-[1.5] ${l.kind === "deploy" ? "text-[var(--red-900)]" : l.kind === "sys" ? "text-white/35" : "text-white/70"}`} style={{ textTransform: "none" }}>
						{l.text}
					</div>
				))}
			</div>
		</div>
	);
}
