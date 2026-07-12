"use client";
import { PixelDecodeText } from "./PixelDecodeText";

// The thesis. produx's about-section is a NORMAL section (not pinned/sticky) — the text scrolls
// naturally up through the viewport while the canvas decode plays over its transit. No "speed bump".
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";

export function ManifestoReveal() {
	return (
		<section className="relative w-full px-[5.5vw] max-lg:px-[4.10vw] max-sm:px-[5.97vw] py-[20vh] z-10">
			<div className="w-full max-w-[82vw] max-lg:max-w-none">
				<PixelDecodeText text={TEXT} />

				<div className="mt-[7vh] max-w-[52ch]">
					<p className="label-13-mono text-[var(--gray-800)] uppercase" style={{ lineHeight: 1.75 }}>
						Escrow that holds. An oracle that checks. A tribunal that rules. Every verdict final,
						on-chain — not a promise, a protocol.
					</p>
					<p className="label-13-mono text-[var(--red-700)] uppercase mt-5 tracking-[0.18em]">
						Built on Casper · Rust/WASM · real settlement.
					</p>
				</div>
			</div>
		</section>
	);
}
