"use client";
import { useEffect, useRef } from "react";

// Port of produx's canvas pixel-decode manifesto (params from their bundle: pixelSize 5,
// revealMode per word, direction bottom->top, Wave Diagonal, edge colour — lime there, red here).
// The glyphs stay SOLID + semi-transparent (dim) and FILL IN smoothly, line by line, left->right.
// Only the moving diagonal FRONT shows pixels (red squares appearing INSIDE the letters, then
// resolving to clean white). Rendered on a canvas -> not selectable, like theirs.
const PIXEL = 5;

export function PixelDecodeText({ text, scrollRef }: { text: string; scrollRef?: React.RefObject<HTMLElement | null> }) {
	const wrapRef = useRef<HTMLDivElement>(null);
	const canvasRef = useRef<HTMLCanvasElement>(null);

	useEffect(() => {
		const canvas = canvasRef.current;
		const wrap = wrapRef.current;
		if (!canvas || !wrap) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;
		const temp = document.createElement("canvas");
		const tctx = temp.getContext("2d");
		if (!tctx) return;

		let W = 0, H = 0, dpr = 1, reveal = 0, target = 0, raf = 0, fs = 0, lineH = 0;
		let off: HTMLCanvasElement | null = null;
		let offData: Uint8ClampedArray | null = null;
		let lines: { y0: number; y1: number; w: number }[] = [];

		const font = (px: number) => `400 ${px}px 'Sora', 'Arial', sans-serif`;
		const hash = (a: number, b: number) => {
			let h = (a * 374761393 + b * 668265263) | 0;
			h = (h ^ (h >> 13)) * 1274126177;
			return ((h ^ (h >> 16)) >>> 0) / 4294967296;
		};

		function layout() {
			dpr = Math.min(window.devicePixelRatio || 1, 1.75);
			W = Math.max(1, Math.floor(wrap!.clientWidth));
			fs = Math.min(Math.max(window.innerWidth * 0.043, 28), 82);
			lineH = fs * 1.16;
			ctx!.font = font(fs);

			const words = text.split(" ");
			const space = ctx!.measureText(" ").width;
			const built: { text: string; w: number }[] = [];
			let cur = "", curW = 0;
			for (const w of words) {
				const ww = ctx!.measureText(w).width;
				const add = (cur ? space : 0) + ww;
				if (cur && curW + add > W) { built.push({ text: cur, w: curW }); cur = w; curW = ww; }
				else { cur += (cur ? " " : "") + w; curW += add; }
			}
			if (cur) built.push({ text: cur, w: curW });
			const nLines = built.length;
			H = Math.ceil(nLines * lineH + fs * 0.45);

			canvas!.width = Math.round(W * dpr);
			canvas!.height = Math.round(H * dpr);
			canvas!.style.width = W + "px";
			canvas!.style.height = H + "px";
			temp.width = canvas!.width;
			temp.height = canvas!.height;

			off = document.createElement("canvas");
			off.width = canvas!.width; off.height = canvas!.height;
			const o = off.getContext("2d")!;
			o.setTransform(dpr, 0, 0, dpr, 0, 0);
			o.font = font(fs); o.textBaseline = "alphabetic"; o.fillStyle = "#fff";
			lines = [];
			built.forEach((ln, k) => {
				const y = fs + k * lineH;
				o.fillText(ln.text, 0, y);
				lines.push({ y0: (y - fs) * dpr, y1: (y + fs * 0.28) * dpr, w: ln.w * dpr });
			});
			offData = o.getImageData(0, 0, off.width, off.height).data;
			draw();
		}

		function draw() {
			if (!off || !offData) return;
			const cw = canvas!.width, ch = canvas!.height;
			ctx!.setTransform(1, 0, 0, 1, 0, 0);
			ctx!.clearRect(0, 0, cw, ch);
			// solid, semi-transparent base — the whole text sits here faintly
			ctx!.globalAlpha = 0.16;
			ctx!.drawImage(off, 0, 0);
			ctx!.globalAlpha = 1;

			const nLines = lines.length;
			const soft = fs * 0.55 * dpr;
			const cell = Math.max(2, Math.round(PIXEL * dpr));

			for (let k = 0; k < nLines; k++) {
				const ln = lines[k];
				const lh = ln.y1 - ln.y0;
				const lp = Math.max(0, Math.min(1, (reveal * nLines - k) * 1.22)); // reading order, slight overlap
				if (lp <= 0) continue;
				const front = lp * (ln.w + soft * 2) - soft;

				// smooth white fill up to the (diagonal) front — this line band only
				tctx!.setTransform(1, 0, 0, 1, 0, 0);
				tctx!.clearRect(0, ln.y0, cw, lh);
				tctx!.globalCompositeOperation = "source-over";
				tctx!.drawImage(off!, 0, ln.y0, cw, lh, 0, ln.y0, cw, lh);
				tctx!.globalCompositeOperation = "destination-in";
				const g = tctx!.createLinearGradient(front - soft, ln.y0, front + soft, ln.y1); // tilt = diagonal
				g.addColorStop(0, "rgba(0,0,0,1)");
				g.addColorStop(1, "rgba(0,0,0,0)");
				tctx!.fillStyle = g;
				tctx!.fillRect(0, ln.y0, cw, lh);
				tctx!.globalCompositeOperation = "source-over";
				ctx!.drawImage(temp, 0, ln.y0, cw, lh, 0, ln.y0, cw, lh);

				// red pixels appearing INSIDE the letters along the front, resolving as it passes
				const bx0 = Math.max(0, front - soft), bx1 = Math.min(cw, front + soft);
				for (let cy = ln.y0; cy < ln.y1; cy += cell) {
					const vert = (cy - ln.y0) / lh; // 0 top .. 1 bottom (bottom resolves first)
					for (let cx = Math.floor(bx0 / cell) * cell; cx < bx1; cx += cell) {
						const sx = Math.min(cw - 1, cx + (cell >> 1));
						const sy = Math.min(ch - 1, cy + (cell >> 1));
						if (offData[(sy * cw + sx) * 4 + 3] < 55) continue; // only inside the glyph
						const gv = (cx - (front - soft)) / (2 * soft); // 0 revealed .. 1 ahead
						if (gv < 0.04 || gv > 0.97) continue;
						const on = hash(Math.round(cx / cell), Math.round(cy / cell)) < (0.3 + 0.62 * gv) * (0.6 + 0.5 * (1 - vert));
						if (on) {
							ctx!.fillStyle = `rgba(241,50,66,${0.5 + 0.45 * gv})`;
							ctx!.fillRect(cx, cy, cell - 1, cell - 1);
						}
					}
				}
			}
		}

		function computeTarget() {
			const st = scrollRef?.current;
			if (st) {
				// tall sticky section (unused now): reveal spans the whole scroll-through.
				const rect = st.getBoundingClientRect();
				const h = Math.max(1, st.offsetHeight - window.innerHeight);
				target = Math.max(0, Math.min(1, (-rect.top / h) / 0.85));
			} else {
				// produx model: NORMAL section, no pin. Decode is tied to the block's natural transit —
				// 0 when it enters from the bottom (top at viewport bottom), completing at ~72% of the
				// way up so the finished text holds white as it keeps scrolling out. Text is never held.
				const r = wrap!.getBoundingClientRect();
				const vh = window.innerHeight;
				const p = (vh - r.top) / (vh + r.height);
				target = Math.max(0, Math.min(1, p / 0.72));
			}
		}

		// scrub-like lag: reveal eases toward the scroll target (~matches the mosaic's scrub feel),
		// so the thesis reads as smooth/heavy as the pinned top instead of snapping 1:1 to scroll.
		function tick() {
			reveal += (target - reveal) * 0.09;
			if (Math.abs(target - reveal) < 0.0006) {
				reveal = target;
				draw();
				raf = 0;
				return;
			}
			draw();
			raf = requestAnimationFrame(tick);
		}

		function onScroll() {
			computeTarget();
			if (!raf) raf = requestAnimationFrame(tick);
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
			<canvas ref={canvasRef} className="block select-none" aria-label={text} />
		</div>
	);
}
