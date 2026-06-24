import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { SmoothScroller } from "@/components/SmoothScroller";
import GlobalBackground from "@/components/GlobalBackground";

const geistSans = Geist({
	variable: "--font-geist-sans",
	subsets: ["latin"],
});

const geistMono = Geist_Mono({
	variable: "--font-geist-mono",
	subsets: ["latin"],
});

export const metadata: Metadata = {
	title: "x402 Arbitrage Mesh — Sovereign Agent Gateway",
	description:
		"Decentralized AI Agent Load Balancer and Payment Router on Stellar Soroban. WASM-sandboxed, L402-secured, zero-trust.",
	openGraph: {
		title: "x402 Arbitrage Mesh",
		description:
			"The world's first Agent-to-Agent Payment Router with WASM Quarantine, built on Stellar.",
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
			className={`${geistSans.variable} ${geistMono.variable} antialiased`}
		>
			<head>
				<link rel="preconnect" href="https://fonts.googleapis.com" />
				<link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
				<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=DM+Sans:wght@400;500;600;700&display=swap" rel="stylesheet" />
			</head>
			<body className="flex flex-col">
				<GlobalBackground />
				<SmoothScroller>{children}</SmoothScroller>
			</body>
		</html>
	);
}
