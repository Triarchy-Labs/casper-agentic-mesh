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



  // GSAP Cinematic Transitions & Pinning (Fable 5 Aesthetic)
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
            <section className="relative h-screen flex items-center justify-center px-8 md:px-16 overflow-hidden">
              <div className="w-full max-w-7xl grid grid-cols-1 md:grid-cols-12 gap-8 items-center relative min-h-[500px]">
                
                {/* Left Column: Heading and CTAs */}
                <div className="md:col-span-5 flex flex-col items-start text-left z-20">
                  <h1 className="heading-72 leading-[1.05] tracking-tight mb-8">
                    Agentic <br/> Infrastructure
                  </h1>
                  <div className="flex gap-[16px]">
                    <button className="button-primary rounded-full px-8 py-3 bg-white text-black border border-white hover:bg-neutral-200 transition-colors">
                      Deploy Now
                    </button>
                    <button className="button-secondary rounded-full px-8 py-3 border border-white/20 text-white bg-black/40 hover:bg-white/10 transition-colors backdrop-blur-md">
                      Talk to Sales
                    </button>
                  </div>
                </div>

                {/* Center Column: Drifting Volumetric Triangle */}
                <div className="md:col-span-4 flex items-center justify-center z-10">
                  <div className="drifting-hero-container">
                    <div className="volumetric-light-cone"></div>
                    <div className="silhouetted-triangle"></div>
                  </div>
                </div>

                {/* Right Column: Mono tags */}
                <div className="md:col-span-3 flex flex-col items-start md:items-end justify-center text-left md:text-right label-14-mono text-white/50 space-y-3 z-20 uppercase tracking-[0.15em] font-medium">
                  <p>FOR CODING AGENTS</p>
                  <p>TO SHIP APPS AND AGENTS</p>
                  <p>AUTOMATED BY AGENTS</p>
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
            <div ref={sectionsRef} className="w-full max-w-6xl mx-auto px-6 py-[96px] flex flex-col gap-[128px]">
              
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
