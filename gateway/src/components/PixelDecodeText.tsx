"use client";
import { useEffect, useRef } from "react";

// Port of produx's canvas pixel-decode text (verified params from their bundle):
//   pixelSize 5 · direction bottom->top · revealMode "per word" · noiseMode "Wave Diagonal"
//   noise .4 · waveRand .4 · edgeColor (theirs #aeff00 lime -> ours brand red) · edge band.
// The glyphs are rasterised to a canvas and revealed cell-by-cell on scroll: a wave sweeps
// word-by-word, each cell flips dim -> RED edge -> white. Not selectable (it's a canvas), like produx.
const PIXEL = 5;
const EDGE = [241, 50, 66]; // #f13242 (was #aeff00 green)
const EDGE_BAND = 0.13; // how long a cell stays "edge red" before settling white

type Cell = { x: number; y: number; th: number };

export function PixelDecodeText({ text }: { text: string }) {
	const wrapRef = useRef<HTMLDivElement>(null);
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const canvas = canvasRef.current;
		const wrap = wrapRef.current;
		if (!canvas || !wrap) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		let cells: Cell[] = [];
		let W = 0, H = 0, dpr = 1, reveal = 0, raf = 0;

		const fontStr = (px: number) => `400 ${px}px var(--font-tech), 'Sora', sans-serif`;

		function layout() {
			dpr = Math.min(window.devicePixelRatio || 1, 2);
			W = Math.max(1, Math.floor(wrap!.clientWidth));
			const fs = Math.min(Math.max(window.innerWidth * 0.044, 28), 86);
			const lineH = fs * 1.14;
			ctx!.font = fontStr(fs);
			const space = ctx!.measureText(" ").width;

			// wrap into lines
			const words = text.split(" ");
			const lines: string[][] = [];
			let line: string[] = [], lw = 0;
			for (const w of words) {
				const ww = ctx!.measureText(w).width;
				if (line.length && lw + space + ww > W) { lines.push(line); line = [w]; lw = ww; }
				else { lw += (line.length ? space : 0) + ww; line.push(w); }
			}
			if (line.length) lines.push(line);
			H = Math.ceil(lines.length * lineH + fs * 0.35);

			canvas!.width = Math.round(W * dpr);
			canvas!.height = Math.round(H * dpr);
			canvas!.style.width = W + "px";
			canvas!.style.height = H + "px";

			// offscreen raster of the white text, remembering word bounds for per-word ordering
			const off = document.createElement("canvas");
			off.width = canvas!.width; off.height = canvas!.height;
			const o = off.getContext("2d")!;
			o.scale(dpr, dpr);
			o.font = fontStr(fs);
			o.textBaseline = "alphabetic";
			o.fillStyle = "#fff";
			const boxes: { x0: number; x1: number; idx: number }[] = [];
			let wi = 0;
			lines.forEach((ln, li) => {
				let x = 0;
				const y = fs + li * lineH;
				for (const w of ln) {
					const ww = o.measureText(w).width;
					o.fillText(w, x, y);
					boxes.push({ x0: x, x1: x + ww, idx: wi++ });
					x += ww + space;
				}
			});
			const totalW = Math.max(1, wi);

			const data = o.getImageData(0, 0, off.width, off.height).data;
			cells = [];
			for (let py = 0; py < H; py += PIXEL) {
				for (let px = 0; px < W; px += PIXEL) {
					const sx = Math.min(off.width - 1, Math.floor((px + PIXEL / 2) * dpr));
					const sy = Math.min(off.height - 1, Math.floor((py + PIXEL / 2) * dpr));
					if (data[(sy * off.width + sx) * 4 + 3] > 45) {
						let widx = 0;
						for (const b of boxes) { if (px >= b.x0 - 3 && px <= b.x1 + 3) { widx = b.idx; break; } }
						const wordT = widx / totalW;               // per word, left -> right (dominant)
						const diag = ((px / W) + (1 - py / H)) * 0.5; // Wave Diagonal, bottom-left leaning
						const th = wordT * 0.72 + diag * 0.08 + Math.random() * 0.18; // noise/waveRand
						cells.push({ x: px, y: py, th: Math.min(0.999, th) });
					}
				}
			}
			draw();
		}

		function draw() {
			ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
			ctx!.clearRect(0, 0, W, H);
			for (let i = 0; i < cells.length; i++) {
				const c = cells[i];
				const d = reveal - c.th;
				if (d >= EDGE_BAND) ctx!.fillStyle = "#ffffff";
				else if (d >= 0) ctx!.fillStyle = `rgb(${EDGE[0]},${EDGE[1]},${EDGE[2]})`;
				else ctx!.fillStyle = "rgba(190,190,196,0.16)";
				ctx!.fillRect(c.x, c.y, PIXEL - 0.6, PIXEL - 0.6);
			}
		}

		function onScroll() {
			if (raf) return;
			raf = requestAnimationFrame(() => {
				raf = 0;
				const r = wrap!.getBoundingClientRect();
				const vh = window.innerHeight;
				const start = vh * 0.82, end = vh * 0.22;
				reveal = Math.max(0, Math.min(1, (start - r.top) / (start - end)));
				draw();
			});
		}

		layout();
		window.addEventListener("resize", layout);
		window.addEventListener("scroll", onScroll, { passive: true });
		if (document.fonts) document.fonts.ready.then(layout).catch(() => {});
		onScroll();
		return () => {
			window.removeEventListener("resize", layout);
			window.removeEventListener("scroll", onScroll);
			if (raf) cancelAnimationFrame(raf);
		};
	}, [text]);

	return (
		<div ref={wrapRef} className="w-full">
			<canvas ref={canvasRef} className="block w-full select-none" aria-label={text} />
		</div>
	);
}
