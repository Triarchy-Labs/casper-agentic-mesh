"use client";

import React from "react";

export function Footer() {
	return (
		<footer className="w-full border-t border-[var(--gray-400)] bg-black py-8 px-6 md:px-16 mt-auto z-20 relative">
			<div className="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-6 label-12-mono text-[var(--gray-700)]">
				<div className="flex flex-col md:flex-row items-center gap-2 md:gap-6">
					<span>© 2026 TRIARCHY LABS</span>
					<span className="hidden md:inline text-[var(--gray-500)]">|</span>
					<span>INSTANTIATED ON CASPER NETWORK</span>
				</div>
				<div className="flex gap-6 items-center flex-wrap justify-center">
					<a
						href="https://casper.network"
						target="_blank"
						rel="noopener noreferrer"
						className="hover:text-[var(--red-700)] transition-colors duration-150"
					>
						[ CASPER.NETWORK ]
					</a>
					<a
						href="https://github.com/make-software/casper-x402"
						target="_blank"
						rel="noopener noreferrer"
						className="hover:text-[var(--red-700)] transition-colors duration-150"
					>
						[ X402 PROTOCOL ]
					</a>
					<a
						href="https://odra.dev"
						target="_blank"
						rel="noopener noreferrer"
						className="hover:text-[var(--red-700)] transition-colors duration-150"
					>
						[ ODRA FRAMEWORK ]
					</a>
				</div>
			</div>
		</footer>
	);
}
