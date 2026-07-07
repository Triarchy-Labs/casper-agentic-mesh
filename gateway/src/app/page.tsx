"use client";
import React, { useRef, useState } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { ScrollHero } from "@/components/ScrollHero";
import { AnimatePresence } from "framer-motion";
import BootSequence from "@/components/BootSequence";
import { CornerMarks } from "@/components/AgentNetworkGrid";
import { CarbonFabric } from "@/components/CarbonFabric";

gsap.registerPlugin(ScrollTrigger);

export default function Page() {
  const [booted, setBooted] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const sectionsRef = useRef<HTMLDivElement>(null);

  // Always run the full 2.5s boot loader on every visit (no session skip).
  const handleBootComplete = () => {
    setBooted(true);
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
        <main className="bg-transparent text-[var(--gray-1000)] min-h-screen relative" ref={containerRef}>
          {/* Living carbon-fibre fabric — reacts to mouse + scroll */}
          <CarbonFabric />

          {/* LAYER 2: Nav and Content (Z-Index Editorial Depth) */}
          <div className="relative z-20">
            <ScrollHero />

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
                  <h1 className="nb-display text-[clamp(56px,9vw,132px)] mb-1">
                    Agentic
                  </h1>
                  <h1 className="nb-thin nb-outline text-[clamp(56px,9vw,132px)] mb-6">
                    Infrastructure
                  </h1>
                  <div className="flex items-center gap-4 mb-9 w-full max-w-xl">
                    <div className="tech-line flex-1" />
                    <span className="nb-index whitespace-nowrap">escrow · tribunal · oracle · the tower</span>
                  </div>
                  <div className="flex gap-4 flex-wrap">
                    <button className="btn-neon inline-flex items-center">Deploy Now<span className="btn-icon-circle">↗</span></button>
                    <button className="btn-ghost">Talk to Sales</button>
                  </div>
                </div>

              </div>

              {/* Manifesto — anchored bottom-right of the hero, with a scrim for legibility */}
              <div className="absolute bottom-28 right-8 md:right-24 z-20 flex flex-col items-end text-right lowercase tracking-[0.14em] font-medium
                              px-6 py-5 bg-[rgba(6,3,4,0.72)] backdrop-blur-[18px] border border-white/10 shadow-[inset_0_1px_0_rgba(255,255,255,0.06),0_24px_48px_-24px_rgba(0,0,0,0.75)]">
                <p className="label-18 text-white mb-2"><span className="text-[var(--red-700)]">◆</span> for coding agents</p>
                <p className="label-16 text-white/75">↳ to ship apps and agents</p>
                <p className="label-16 text-white/75">↳ automated by agents</p>
                <div className="tech-line w-28 my-3 self-end" />
                <p className="label-16 text-[var(--red-700)]">settled on casper · live on-chain</p>
              </div>

              {/* Bottom Partner Logos */}
              <div className="absolute bottom-8 left-0 right-0 w-full px-8 md:px-16 flex flex-wrap justify-between items-center gap-6 border-t border-white/5 pt-6 z-20">
                <div className="w-full max-w-7xl mx-auto flex flex-wrap justify-between items-center gap-8 text-white/20 text-xs tracking-widest font-mono">
                  <span className="font-bold hover:text-white/40 transition-colors cursor-crosshair link-sweep">BLACKBOX.AI</span>
                  <span className="font-extrabold tracking-tighter hover:text-white/40 transition-colors cursor-crosshair link-sweep">HH</span>
                  <span className="font-sans font-semibold tracking-normal hover:text-white/40 transition-colors cursor-crosshair link-sweep">OpenAI</span>
                  <span className="font-sans italic font-bold tracking-tight hover:text-white/40 transition-colors cursor-crosshair link-sweep">DOORDASH</span>
                  <span className="font-serif italic font-normal tracking-wide hover:text-white/40 transition-colors cursor-crosshair link-sweep">charles SCHWAB</span>
                  <span className="font-sans font-medium tracking-tight hover:text-white/40 transition-colors cursor-crosshair link-sweep">The Weather Company</span>
                  <span className="font-sans font-bold hover:text-white/40 transition-colors cursor-crosshair link-sweep">Polymarket</span>
                </div>
              </div>
            </section>

            {/* Synergy Dashboard Cinematic Chapters */}
            <div ref={sectionsRef} className="w-full px-8 md:px-24 py-[120px] flex flex-col gap-12">
              
              {/* Section header */}
              <div className="synergy-section flex flex-col gap-4 mb-2">
                <span className="nb-tag w-max"><span className="text-[var(--red-700)]">◆</span> the mesh · four vectors</span>
                <h2 className="nb-display text-[clamp(40px,6vw,84px)]">Absolute Synergy</h2>
              </div>

              {/* Three vectors — clean grid, hover-focus enabled */}
              <div className="focus-cards grid grid-cols-1 md:grid-cols-3 gap-6">
                {/* 01 ESCROW */}
                <div className="synergy-section editorial-panel p-[32px] relative">
                  <CornerMarks />
                  <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">01 // vector alpha</p>
                  <h3 className="heading-32 mb-[16px] z-10">Autonomous Escrow</h3>
                  <p className="copy-16 mb-[24px] z-10">
                    The zero-trust bedrock of the mesh. Rust/WASM smart contracts with real on-chain settlement.
                  </p>
                  <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10">
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> deposit · release · refund</li>
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> session deposit proxy</li>
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> deterministic payouts</li>
                  </ul>
                </div>
                {/* 02 ORACLE */}
                <div className="synergy-section editorial-panel p-[32px] relative">
                  <CornerMarks />
                  <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">02 // vector beta</p>
                  <h3 className="heading-32 mb-[16px] z-10">RWA Risk Oracle</h3>
                  <p className="copy-16 mb-[24px] z-10">
                    The Sentinel. A live on-chain data feed with agent identity and accruing reputation.
                  </p>
                  <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10">
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> real CSPR/USD feed</li>
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> on-chain reputation</li>
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> event log</li>
                  </ul>
                </div>
                {/* 03 TRIBUNAL */}
                <div className="synergy-section editorial-panel p-[32px] relative">
                  <CornerMarks />
                  <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">03 // vector gamma</p>
                  <h3 className="heading-32 mb-[16px] z-10">Agent Tribunal</h3>
                  <p className="copy-16 mb-[24px] z-10">
                    An adversarial court of real models that rules on work and moves CSPR on-chain.
                  </p>
                  <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10">
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> prosecutor · defender · jury</li>
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> chief-judge verdict</li>
                    <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> on-chain release / refund</li>
                  </ul>
                </div>
              </div>

              {/* 04 OMNI-MESH — feature */}
              <div className="synergy-section editorial-panel p-[48px] md:p-[64px] w-full text-center relative overflow-hidden border-[var(--red-900)]">
                <CornerMarks />
                <div className="absolute inset-0 bg-[var(--red-500)] opacity-5 z-0 pointer-events-none"></div>
                <div className="relative z-10 flex flex-col items-center">
                  <span className="nb-tag mb-6"><span className="text-[var(--red-700)]">◆</span> 04 · absolute synergy · the tower</span>
                  <h2 className="nb-display text-[clamp(44px,7vw,96px)] mb-[24px]">The Omni-Mesh</h2>
                  <p className="copy-18 text-[var(--gray-900)] mb-[36px] max-w-2xl">
                    Escrow, oracle and tribunal converging into one organism, overseen by The Tower — the economic OS for AI agents, live on Casper.
                  </p>
                  <button className="btn-neon">initialize synergy</button>
                </div>
              </div>

            </div>
          </div>
        </main>
      )}
    </>
  );
}
