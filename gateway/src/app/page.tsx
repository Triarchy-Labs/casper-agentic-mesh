"use client";
import React, { useRef, useEffect, useState } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { Nav } from "@/components/Nav";
import FluidBackground from "@/components/FluidBackground";
import { AnimatePresence } from "framer-motion";
import BootSequence from "@/components/BootSequence";
import { CornerMarks } from "@/components/AgentNetworkGrid";

gsap.registerPlugin(ScrollTrigger);

export default function Page() {
  const [booted, setBooted] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const sectionsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (typeof window !== "undefined") {
      const isBooted = sessionStorage.getItem("x402_booted") === "true";
      if (isBooted) {
        setTimeout(() => setBooted(true), 0);
      }
    }
  }, []);

  const handleBootComplete = () => {
    setBooted(true);
    if (typeof window !== "undefined") {
      sessionStorage.setItem("x402_booted", "true");
    }
  };



  // GSAP Cinematic Transitions & Pinning (Brutalist Aesthetic)
  useGSAP(() => {
    const sections = gsap.utils.toArray(".synergy-section") as HTMLElement[];
    sections.forEach((section) => {
      gsap.fromTo(section, 
        { opacity: 0, y: 100, scale: 0.95 },
        {
          opacity: 1, 
          y: 0,
          scale: 1,
          duration: 1,
          ease: "power3.out",
          scrollTrigger: {
            trigger: section,
            start: "top 80%",
            end: "center center",
            scrub: 0.5, 
            toggleActions: "play reverse play reverse",
          }
        }
      );
    });
  }, { scope: containerRef });

  return (
    <>
      <AnimatePresence mode="wait">
        {!booted && <BootSequence onComplete={handleBootComplete} />}
      </AnimatePresence>

      {booted && (
        <main className="bg-[var(--background-100)] text-[var(--gray-1000)] min-h-screen relative" ref={containerRef}>
          {/* LAYER 0: Fluid Three.js Tissue (Reacts to cursor) */}
          <FluidBackground />

          {/* LAYER 2: Nav and Content (Z-Index Editorial Depth) */}
          <div className="relative z-20">
            <Nav />

            {/* Hero Section */}
            <section className="relative h-screen flex items-center px-8 md:px-24 overflow-hidden">
              <div className="w-full grid grid-cols-1 md:grid-cols-12 gap-12 items-center relative min-h-[500px]">

                {/* Left: Heading and CTAs (full-bleed, breathing) */}
                <div className="md:col-span-8 md:col-start-1 flex flex-col items-start text-left z-20 pl-2 md:pl-12">
                  <div className="flex items-center gap-3 mb-7 flex-wrap">
                    <span className="nb-tag"><span className="text-[var(--red-700)]">◆</span> casper · testnet live</span>
                    <span className="nb-tag nb-tag-ghost">/// vol.𝟎𝟏 — agent economy</span>
                    <span className="nb-index">𝟐𝟎𝟐𝟔</span>
                  </div>
                  <h1 className="nb-display text-[64px] md:text-[112px] mb-1">
                    Agentic
                  </h1>
                  <h1 className="nb-thin nb-outline text-[64px] md:text-[112px] mb-6">
                    Infrastructure
                  </h1>
                  <div className="flex items-center gap-4 mb-9 w-full max-w-xl">
                    <div className="tech-line flex-1" />
                    <span className="nb-index whitespace-nowrap">escrow · tribunal · oracle · the tower</span>
                  </div>
                  <div className="flex gap-4 flex-wrap">
                    <button className="btn-neon">Deploy Now</button>
                    <button className="btn-ghost">Talk to Sales</button>
                  </div>
                </div>

                {/* Right: Mono manifesto tags */}
                <div className="md:col-span-3 md:col-start-10 flex flex-col items-start md:items-end justify-center text-left md:text-right label-14-mono text-white/45 space-y-3 z-20 lowercase tracking-[0.18em] font-medium">
                  <p className="text-white/70">◆ for coding agents</p>
                  <p>↳ to ship apps and agents</p>
                  <p>↳ automated by agents</p>
                  <div className="tech-line w-24 my-2 self-end" />
                  <p className="text-[var(--red-700)]">settled on casper · live on-chain</p>
                </div>

              </div>

              {/* Bottom Partner Logos */}
              <div className="absolute bottom-8 left-0 right-0 w-full px-8 md:px-16 flex flex-wrap justify-between items-center gap-6 border-t border-white/5 pt-6 z-20">
                <div className="w-full max-w-7xl mx-auto flex flex-wrap justify-between items-center gap-8 text-white/20 text-xs tracking-widest font-mono">
                  <span className="font-bold hover:text-white/40 transition-colors cursor-crosshair">BLACKBOX.AI</span>
                  <span className="font-extrabold tracking-tighter hover:text-white/40 transition-colors cursor-crosshair">HH</span>
                  <span className="font-sans font-semibold tracking-normal hover:text-white/40 transition-colors cursor-crosshair">OpenAI</span>
                  <span className="font-sans italic font-bold tracking-tight hover:text-white/40 transition-colors cursor-crosshair">DOORDASH</span>
                  <span className="font-serif italic font-normal tracking-wide hover:text-white/40 transition-colors cursor-crosshair">charles SCHWAB</span>
                  <span className="font-sans font-medium tracking-tight hover:text-white/40 transition-colors cursor-crosshair">The Weather Company</span>
                  <span className="font-sans font-bold hover:text-white/40 transition-colors cursor-crosshair">Polymarket</span>
                </div>
              </div>
            </section>

            {/* Synergy Dashboard Cinematic Chapters */}
            <div ref={sectionsRef} className="w-full px-8 md:px-24 py-[96px] flex flex-col gap-[128px]">
              
              {/* TAB 1: ESCROW */}
              <div className="synergy-section min-h-[60vh] flex items-center justify-start sticky top-[20vh]">
                <div className="editorial-panel p-[40px] w-full max-w-lg relative">
                  <CornerMarks />
                  <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">01 // Vector Alpha</p>
                  <h2 className="heading-40 mb-[16px] z-10">Autonomous Escrow</h2>
                  <p className="copy-16 mb-[24px] z-10">
                    The zero-trust bedrock of the Agentic Mesh. Powered by the <strong>Odra Framework</strong> for high-security Rust/WASM smart contracts.
                  </p>
                  <ul className="label-14-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10">
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> casper-eip-712 Meta-Transactions
                    </li>
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> Deterministic Payouts
                    </li>
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> Casper Wallet Override
                    </li>
                  </ul>
                </div>
              </div>

              {/* TAB 2: ORACLE */}
              <div className="synergy-section min-h-[60vh] flex items-center justify-end sticky top-[30vh]">
                <div className="editorial-panel p-[40px] w-full max-w-lg relative">
                  <CornerMarks />
                  <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">02 // Vector Beta</p>
                  <h2 className="heading-40 mb-[16px] z-10">Pre-Trade Risk Oracle</h2>
                  <p className="copy-16 mb-[24px] z-10">
                    The Sentinel. An autonomous swarm evaluating risk, sentiment, and complexity prior to escrow lock.
                  </p>
                  <ul className="label-14-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10">
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> Casper MCP Server
                    </li>
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> Live On-Chain Reconnaissance
                    </li>
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> LLM Consensus Engine
                    </li>
                  </ul>
                </div>
              </div>

              {/* TAB 3: ARBITRAGE */}
              <div className="synergy-section min-h-[60vh] flex items-center justify-start sticky top-[40vh]">
                <div className="editorial-panel p-[40px] w-full max-w-lg relative">
                  <CornerMarks />
                  <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">03 // Vector Gamma</p>
                  <h2 className="heading-40 mb-[16px] z-10">Cognitive Arbitrage</h2>
                  <p className="copy-16 mb-[24px] z-10">
                    The Predator. Hunting mispriced high-value tasks across the mesh using autonomous M2M commerce.
                  </p>
                  <ul className="label-14-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10">
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> x402 Facilitator (Go Sidecar)
                    </li>
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> CEP-18 Micropayments
                    </li>
                    <li className="flex items-center gap-[8px]">
                      <span className="text-[var(--red-700)]">→</span> Odra X402Liquidator WASM
                    </li>
                  </ul>
                </div>
              </div>

              {/* TAB 4: SYNERGY */}
              <div className="synergy-section min-h-screen flex items-center justify-center relative">
                <div className="editorial-panel p-[48px] w-full max-w-3xl text-center border-[var(--red-900)] relative overflow-hidden">
                  <CornerMarks />
                  <div className="absolute inset-0 bg-[var(--red-500)] opacity-5 z-0 pointer-events-none"></div>
                  <div className="relative z-10">
                    <p className="label-14-mono text-[var(--red-700)] mb-[16px]">04 // Absolute Synergy</p>
                    <h2 className="heading-56 mb-[24px] text-gradient-casper">The Omni-Mesh</h2>
                    <p className="copy-16 text-[var(--gray-900)] mb-[32px] max-w-2xl mx-auto">
                      Three distinct predators converging into a single, unstoppable biome. The ultimate M2M economy, completely built on Casper Network primitives.
                    </p>
                    <button className="button-primary" style={{ padding: '0 24px', height: '48px', fontSize: '16px' }}>
                      INITIALIZE SYNERGY
                    </button>
                  </div>
                </div>
              </div>

            </div>
          </div>
        </main>
      )}
    </>
  );
}
