"use client";
import { useRef } from "react";
import { PixelDecodeText } from "./PixelDecodeText";

// The thesis — bridge from the hero's emotion + pillar cards to conviction.
// Rendered with produx's canvas pixel-decode. Lives in a TALL sticky section so the
// decode plays over ~2-3 wheel scrolls with the text held in view (like produx), instead
// of flying by in one gesture — which also evens out the scroll cadence after the pinned mosaic.
const TEXT =
	"Everyone is racing to make machines more autonomous. We build the part that makes autonomy safe to trust — accountability, written in code, settled on Casper.";

export function ManifestoReveal() {
	const sectionRef = useRef<HTMLElement>(null);
	return (
		<section ref={sectionRef} className="relative w-full min-h-[300vh] z-10">
			<div className="sticky top-0 h-screen w-full flex flex-col justify-center px-[5.5vw] max-lg:px-[4.10vw] max-sm:px-[5.97vw]">
				<div className="w-full max-w-[82vw] max-lg:max-w-none">
					<PixelDecodeText text={TEXT} scrollRef={sectionRef} />

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
			</div>
		</section>
	);
}
