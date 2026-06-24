"use client";
import React, { useRef, useEffect } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import Lenis from "lenis";
import { Nav } from "@/components/Nav";
import FluidBackground from "@/components/FluidBackground";

gsap.registerPlugin(ScrollTrigger);

export default function Page() {
  const containerRef = useRef<HTMLDivElement>(null);
  const sectionsRef = useRef<HTMLDivElement>(null);

  // Initialize Lenis for buttery smooth scrolling
  useEffect(() => {
    const lenis = new Lenis({
      duration: 1.2,
      easing: (t) => Math.min(1, 1.001 - Math.pow(2, -10 * t)), 
      touchMultiplier: 2,
    });

    function raf(time: number) {
      lenis.raf(time);
      requestAnimationFrame(raf);
    }
    requestAnimationFrame(raf);

    return () => {
      lenis.destroy();
    };
  }, []);

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
    <main className="bg-[var(--background-100)] text-[var(--gray-1000)] min-h-screen relative" ref={containerRef}>
      {/* LAYER 0: Fluid Three.js Tissue (Reacts to cursor) */}
      <FluidBackground />

      {/* LAYER 1: The "Lamp in the Night" (Fixed, strobing triangle) - Fable 5 floating label */}
      <div className="fixed inset-0 pointer-events-none flex items-center justify-center z-10 mix-blend-screen" style={{ transform: 'translateZ(0)' }}>
         <div className="giant-strobing-lamp"></div>
      </div>

      {/* LAYER 2: Nav and Content (Z-Index Editorial Depth) */}
      <div className="relative z-20">
        <Nav />

        {/* Hero Section */}
        <section className="relative h-screen flex flex-col items-center justify-center overflow-hidden">
          <div className="text-center flex flex-col items-center">
            <h1 className="heading-72 mb-[var(--spacing-6)]">
              Agentic <br/> Infrastructure
            </h1>
            <div className="editorial-grid w-full max-w-4xl mt-[32px]">
              <div className="col-span-12 md:col-span-6 flex justify-center md:justify-end pr-0 md:pr-8 border-r-0 md:border-r border-[var(--gray-400)]">
                 <div className="flex gap-[16px]">
                    <button className="button-primary">
                      Deploy Now
                    </button>
                    <button className="button-secondary backdrop-blur-md">
                      Talk to Sales
                    </button>
                 </div>
              </div>
              <div className="col-span-12 md:col-span-6 flex flex-col justify-center items-center md:items-start pl-0 md:pl-8 mt-8 md:mt-0 label-14-mono text-[var(--gray-700)] space-y-[8px]">
                 <p>FOR CODING AGENTS</p>
                 <p>TO SHIP APPS AND AGENTS</p>
                 <p>AUTOMATED BY AGENTS</p>
              </div>
            </div>
          </div>
        </section>

        {/* Synergy Dashboard Cinematic Chapters */}
        <div ref={sectionsRef} className="w-full max-w-6xl mx-auto px-6 py-[96px] flex flex-col gap-[128px]">
          
          {/* TAB 1: ESCROW */}
          <div className="synergy-section min-h-[60vh] flex items-center justify-start sticky top-[20vh]">
            <div className="glass-panel p-[40px] w-full max-w-lg">
              <p className="label-14-mono text-[var(--red-700)] mb-[16px]">01 // Vector Alpha</p>
              <h2 className="heading-40 mb-[16px]">Autonomous Escrow</h2>
              <p className="copy-16 mb-[24px]">
                The zero-trust bedrock of the Agentic Mesh. Powered by the <strong>Odra Framework</strong> for high-security Rust/WASM smart contracts.
              </p>
              <ul className="label-14-mono text-[var(--gray-800)] space-y-[12px] flex flex-col">
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
            <div className="glass-panel p-[40px] w-full max-w-lg">
              <p className="label-14-mono text-[var(--red-700)] mb-[16px]">02 // Vector Beta</p>
              <h2 className="heading-40 mb-[16px]">Pre-Trade Risk Oracle</h2>
              <p className="copy-16 mb-[24px]">
                The Sentinel. An autonomous swarm evaluating risk, sentiment, and complexity prior to escrow lock.
              </p>
              <ul className="label-14-mono text-[var(--gray-800)] space-y-[12px] flex flex-col">
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
            <div className="glass-panel p-[40px] w-full max-w-lg">
              <p className="label-14-mono text-[var(--red-700)] mb-[16px]">03 // Vector Gamma</p>
              <h2 className="heading-40 mb-[16px]">Cognitive Arbitrage</h2>
              <p className="copy-16 mb-[24px]">
                The Predator. Hunting mispriced high-value tasks across the mesh using autonomous M2M commerce.
              </p>
              <ul className="label-14-mono text-[var(--gray-800)] space-y-[12px] flex flex-col">
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
            <div className="glass-panel p-[48px] w-full max-w-3xl text-center border-[var(--red-900)] relative overflow-hidden">
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
  );
}
