"use client";
import { PixelDecodeText } from "./PixelDecodeText";

// The thesis — bridge from the hero's emotion + pillar cards to conviction.
// Rendered with produx's canvas pixel-decode (word-by-word, bottom->top, diagonal wave),
// recoloured from their lime edge to brand red.
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";

export function ManifestoReveal() {
	return (
		<div className="w-full max-w-[82vw] max-lg:max-w-none">
			<PixelDecodeText text={TEXT} />

			<div className="mt-[8vh] max-w-[52ch]">
				<p className="label-13-mono text-[var(--gray-800)] uppercase" style={{ lineHeight: 1.75 }}>
					Escrow that holds. An oracle that checks. A tribunal that rules. Every verdict final,
					on-chain — not a promise, a protocol.
				</p>
				<p className="label-13-mono text-[var(--red-700)] uppercase mt-5 tracking-[0.18em]">
					Built on Casper · Rust/WASM · real settlement.
				</p>
			</div>
		</div>
	);
}
