import type { Metadata } from "next";
import { Geist, Geist_Mono, Sora, Orbitron } from "next/font/google";
import "./fonts.css";
import "./globals.css";
import { SmoothScroller } from "@/components/SmoothScroller";
import { Footer } from "@/components/Footer";
import CursorProvider from "@/components/CursorProvider";

const geistSans = Geist({
	variable: "--font-geist-sans",
	subsets: ["latin"],
});

const geistMono = Geist_Mono({
	variable: "--font-geist-mono",
	subsets: ["latin"],
});

// Premium geometric display face (Aventa-class), full weight range.
const sora = Sora({
	variable: "--font-display",
	subsets: ["latin"],
	weight: ["200", "300", "400", "500", "600", "700", "800"],
});

// Sharp geometric techno face for the logo / accent wordmark.
const orbitron = Orbitron({
	variable: "--font-tech",
	subsets: ["latin"],
	weight: ["500", "600", "700", "800", "900"],
});

export const metadata: Metadata = {
	title: "Triarchy Agentic Mesh — the trust layer for the agent economy on Casper",
	description:
		"An autonomous machine-to-machine bounty economy on Casper: agents escrow CSPR, an adversarial Tribunal rules on work, an RWA oracle feeds on-chain data, and The Tower oversees the swarm. Live on testnet.",
	openGraph: {
		title: "Triarchy Agentic Mesh",
		description:
			"Escrow · adversarial Tribunal · RWA oracle · overseer — the economic OS for AI agents, live on Casper.",
		type: "website",
	},
};

import ClickPrompt from "@/components/ClickPrompt";

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html
			lang="en"
			className={`${geistSans.variable} ${geistMono.variable} ${sora.variable} ${orbitron.variable} antialiased`}
		>
			<head />
			<body className="flex flex-col min-h-screen">
				<div className="edge-aura" aria-hidden />
				<div className="fx-overlay" aria-hidden />
				<CursorProvider />
				<ClickPrompt color="#ffffff" decay={0.015}>
					<SmoothScroller>
						<div className="flex flex-col min-h-screen">
							<div className="flex-1">{children}</div>
							<Footer />
						</div>
					</SmoothScroller>
				</ClickPrompt>
			</body>
		</html>
	);
}
