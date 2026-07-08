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
import { CinematicDim } from "@/components/CinematicDim";

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
          <CinematicDim />

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
                  <h1 className="nb-display text-[clamp(64px,11vw,700px)] mb-1">
                    Agentic
                  </h1>
                  <h1 className="nb-thin nb-outline text-[clamp(64px,11vw,700px)] mb-6">
                    Infrastructure
                  </h1>
                  <div className="flex items-center gap-4 mb-9 w-full max-w-xl">
                    <div className="tech-line flex-1" />
                    <span className="nb-index whitespace-nowrap">escrow · tribunal · oracle · the tower</span>
                  </div>
                  <div className="flex gap-4 flex-wrap">
                    <button className="button-primary inline-flex items-center">Deploy Now<span className="btn-icon-circle ml-2">↗</span></button>
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
            <div ref={sectionsRef} className="w-full px-[5.5vw] py-[120px] flex flex-col gap-12">
              
              {/* Section header */}
              <div className="synergy-section flex flex-col gap-4 mb-2">
                <span className="nb-tag w-max"><span className="text-[var(--red-700)]">◆</span> the mesh · five vectors</span>
                <h2 className="nb-display text-[clamp(40px,6vw,84px)]">Absolute Synergy</h2>
              </div>

              {/* Produx-style: 3 grid rows, flex-col gap-[13.67vh] between rows (from dump) */}
              <div className="flex flex-col gap-[13.67vh] max-sm:gap-[6vh] pt-[11.7vh] max-sm:pt-[4vh]">

                {/* Row 1: big-left (7) + small-right-down (4) */}
                <div className="focus-cards grid grid-cols-12 gap-[1.39vw] max-sm:grid-cols-1 max-sm:gap-[3.48vh]">
                  {/* 01 ESCROW — col-span-7, big left */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-7 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh]">
                      <CornerMarks />
                      <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">01 // vector alpha</p>
                      <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10 mt-auto">
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> deposit · release · refund</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> session deposit proxy</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> deterministic payouts</li>
                      </ul>
                      <div className="panel-cta z-10">
                        <span className="panel-cta-square" />
                        <span className="panel-cta-text">deploy escrow</span>
                      </div>
                      <div className="panel-tags absolute bottom-6 right-6 flex gap-2 z-20">
                        <span className="nb-tag">Escrow</span>
                        <span className="nb-tag">Proxy</span>
                        <span className="nb-tag">Deterministic</span>
                      </div>
                    </div>
                    <div className="project-info-trigger relative flex w-full">
                      <div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="heading-32 leading-tight">Autonomous Escrow</h3>
                        <p className="label-13-mono text-[var(--gray-800)] uppercase">The zero-trust bedrock of the mesh. Rust/WASM smart contracts with real on-chain settlement.</p>
                      </div>
                    </div>
                  </div>

                  {/* 02 ORACLE — col-span-4, right, pushed down (mt-auto) */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-4 md:col-end-13 md:mt-auto cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative">
                      <CornerMarks />
                      <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">02 // vector beta</p>
                      <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10 mt-auto">
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> real CSPR/USD feed</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> on-chain reputation</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> event log</li>
                      </ul>
                      <div className="panel-cta z-10">
                        <span className="panel-cta-square" />
                        <span className="panel-cta-text">query oracle</span>
                      </div>
                      <div className="panel-tags absolute bottom-6 right-6 flex gap-2 z-20">
                        <span className="nb-tag">Oracle</span>
                        <span className="nb-tag">Price Feed</span>
                        <span className="nb-tag">Reputation</span>
                      </div>
                    </div>
                    <div className="project-info-trigger relative flex w-full">
                      <div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="heading-32 leading-tight">RWA Risk Oracle</h3>
                        <p className="label-13-mono text-[var(--gray-800)] uppercase">The Sentinel. A live on-chain data feed with agent identity and accruing reputation.</p>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Row 2: wide centered (10) — the tower */}
                <div className="grid grid-cols-12">
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-10 md:col-start-2 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[3.5vw] md:p-[4.5vw] relative overflow-hidden border-[var(--red-900)] text-center min-h-[48.8vh] flex flex-col justify-center">
                      <CornerMarks />
                      <div className="absolute inset-0 bg-[var(--red-500)] opacity-5 z-0 pointer-events-none"></div>
                      <div className="relative z-10 flex flex-col items-center">
                        <span className="nb-tag mb-6"><span className="text-[var(--red-700)]">◆</span> 03 · absolute synergy · the tower</span>
                        <h2 className="nb-display text-[clamp(44px,7vw,96px)] mb-[24px]">The Omni-Mesh</h2>
                        <p className="copy-18 text-[var(--gray-900)] mb-[36px] max-w-2xl">
                          Escrow, oracle and tribunal converging into one organism, overseen by The Tower — the economic OS for AI agents, live on Casper.
                        </p>
                        <button className="button-primary">INITIALIZE SYNERGY</button>
                      </div>
                      <div className="panel-tags absolute bottom-6 right-6 flex gap-2 z-20">
                        <span className="nb-tag">Omni-Mesh</span>
                        <span className="nb-tag">The Tower</span>
                        <span className="nb-tag">Liveness</span>
                      </div>
                    </div>
                    <div className="project-info-trigger relative flex w-full">
                      <div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="heading-32 leading-tight">The Omni-Mesh</h3>
                        <p className="label-13-mono text-[var(--gray-800)] uppercase">The economic OS for AI agents. All vectors converging into one organism.</p>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Row 3: small-left (4) + big-right (7) */}
                <div className="focus-cards grid grid-cols-12 gap-[1.39vw] max-sm:grid-cols-1 max-sm:gap-[3.48vh]">
                  {/* 04 TRIBUNAL — col-span-4, small left */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-4 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative">
                      <CornerMarks />
                      <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">04 // vector gamma</p>
                      <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10 mt-auto">
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> prosecutor · defender · jury</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> chief-judge verdict</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> on-chain release / refund</li>
                      </ul>
                      <div className="panel-cta z-10">
                        <span className="panel-cta-square" />
                        <span className="panel-cta-text">view tribunal</span>
                      </div>
                      <div className="panel-tags absolute bottom-6 right-6 flex gap-2 z-20">
                        <span className="nb-tag">Tribunal</span>
                        <span className="nb-tag">Chief Judge</span>
                        <span className="nb-tag">Adversarial</span>
                      </div>
                    </div>
                    <div className="project-info-trigger relative flex w-full">
                      <div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="heading-32 leading-tight">Agent Tribunal</h3>
                        <p className="label-13-mono text-[var(--gray-800)] uppercase">An adversarial court of real models that rules on work and moves CSPR on-chain.</p>
                      </div>
                    </div>
                  </div>

                  {/* 05 x402 PAYMENT — col-span-7, big right */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-7 md:col-end-13 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh]">
                      <CornerMarks />
                      <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">05 // vector delta</p>
                      <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10 mt-auto">
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> pay-per-call micro-settlements</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> HTTP 402 protocol gateway</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> agent wallet abstraction</li>
                      </ul>
                      <div className="panel-cta z-10">
                        <span className="panel-cta-square" />
                        <span className="panel-cta-text">explore x402</span>
                      </div>
                      <div className="panel-tags absolute bottom-6 right-6 flex gap-2 z-20">
                        <span className="nb-tag">x402</span>
                        <span className="nb-tag">Micro-payments</span>
                        <span className="nb-tag">Gateway</span>
                      </div>
                    </div>
                    <div className="project-info-trigger relative flex w-full">
                      <div className="my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="heading-32 leading-tight">x402 Payment Layer</h3>
                        <p className="label-13-mono text-[var(--gray-800)] uppercase">HTTP 402 native. Agents pay per API call with real CSPR — no subscriptions, no keys, just value for value.</p>
                      </div>
                    </div>
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
