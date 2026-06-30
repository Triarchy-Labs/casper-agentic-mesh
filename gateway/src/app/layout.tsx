import type { Metadata } from "next";
import { Geist, Geist_Mono, Sora } from "next/font/google";
import "./globals.css";
import { SmoothScroller } from "@/components/SmoothScroller";
import { Footer } from "@/components/Footer";
import { CursorAura } from "@/components/CursorAura";

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

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html
			lang="en"
			className={`${geistSans.variable} ${geistMono.variable} ${sora.variable} antialiased`}
		>
			<head />
			<body className="flex flex-col min-h-screen">
				<div className="edge-aura" aria-hidden />
				<CursorAura />
				<SmoothScroller>
					<div className="flex flex-col min-h-screen">
						<div className="flex-1">{children}</div>
						<Footer />
					</div>
				</SmoothScroller>
			</body>
		</html>
	);
}
