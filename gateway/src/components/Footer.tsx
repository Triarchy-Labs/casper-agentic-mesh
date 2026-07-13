"use client";

import React from "react";
import { usePathname } from "next/navigation";

export function Footer() {
	// The home page carries the full MeshFooter (produx skeleton + CASPER anchor); this telemetry
	// strip stays for inner pages only, its phrases were merged into MeshFooter's bottom bar.
	const pathname = usePathname();
	if (pathname === "/") return null;
	return (
		<footer className="w-full border-t border-white/10 bg-black/40 backdrop-blur-md py-8 px-6 md:px-16 mt-auto z-20 relative">
			<div className="max-w-7xl mx-auto">
				{/* industrial telemetry strip */}
				<div className="flex items-center gap-4 mb-6 label-12-mono text-[var(--gray-600)]">
					<span className="text-[var(--red-700)]">///</span>
					<span>REV 2.6</span>
					<span className="text-[var(--gray-500)]">·</span>
					<span>UNIT / MESH-01</span>
					<span className="text-[var(--gray-500)]">·</span>
					<span>BUILD 2026.07</span>
					<span className="text-[var(--gray-500)]">·</span>
					<span>TRIARCHY&trade;</span>
					<div className="hr-brutal flex-1 ml-2" />
				</div>

				<div className="flex flex-col md:flex-row justify-between items-center gap-6 label-12-mono text-[var(--gray-700)]">
					<div className="flex flex-col md:flex-row items-center gap-2 md:gap-6">
						<span>&copy; 2026 TRIARCHY LABS &reg;</span>
						<span className="hidden md:inline text-[var(--gray-500)]">|</span>
						<span>INSTANTIATED ON CASPER NETWORK</span>
					</div>
					<div className="flex gap-6 items-center flex-wrap justify-center">
						<a href="https://casper.network" target="_blank" rel="noopener noreferrer" className="link-sweep">[ CASPER.NETWORK ]</a>
						<a href="https://github.com/Triarchy-Labs/casper-agentic-mesh" target="_blank" rel="noopener noreferrer" className="link-sweep">[ GITHUB ]</a>
						<a href="https://odra.dev" target="_blank" rel="noopener noreferrer" className="link-sweep">[ ODRA FRAMEWORK ]</a>
					</div>
				</div>
			</div>
		</footer>
	);
}
