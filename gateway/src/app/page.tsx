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

  // 12-piece mosaic assembly specifications (4 cols x 3 rows)
  // Some pieces have initialOpacity: 0.8 to be visible at their scattered positions on load
  const pieces = [
    // Row 1
    { id: 1, inset: "inset(0% 75% 66.66% 0%)", x: "-25vw", y: "-25vh", scale: 0.8, blur: "6px", initialOpacity: 0.8 },
    { id: 2, inset: "inset(0% 50% 66.66% 25%)", x: "-5vw", y: "-30vh", scale: 0.7, blur: "4px", initialOpacity: 0 },
    { id: 3, inset: "inset(0% 25% 66.66% 50%)", x: "10vw", y: "-28vh", scale: 0.9, blur: "8px", initialOpacity: 0.8 },
    { id: 4, inset: "inset(0% 0% 66.66% 75%)", x: "30vw", y: "-20vh", scale: 0.85, blur: "5px", initialOpacity: 0 },
    // Row 2
    { id: 5, inset: "inset(33.33% 75% 33.33% 0%)", x: "-32vw", y: "-5vh", scale: 0.75, blur: "7px", initialOpacity: 0.8 },
    { id: 6, inset: "inset(33.33% 50% 33.33% 25%)", x: "-12vw", y: "15vh", scale: 0.8, blur: "5px", initialOpacity: 0 },
    { id: 7, inset: "inset(33.33% 25% 33.33% 50%)", x: "15vw", y: "-10vh", scale: 0.9, blur: "6px", initialOpacity: 0.8 },
    { id: 8, inset: "inset(33.33% 0% 33.33% 75%)", x: "28vw", y: "5vh", scale: 0.8, blur: "4px", initialOpacity: 0 },
    // Row 3
    { id: 9, inset: "inset(66.66% 75% 0% 0%)", x: "-28vw", y: "22vh", scale: 0.85, blur: "6px", initialOpacity: 0.8 },
    { id: 10, inset: "inset(66.66% 50% 0% 25%)", x: "-8vw", y: "28vh", scale: 0.7, blur: "8px", initialOpacity: 0 },
    { id: 11, inset: "inset(66.66% 25% 0% 50%)", x: "8vw", y: "25vh", scale: 0.75, blur: "5px", initialOpacity: 0.8 },
    { id: 12, inset: "inset(66.66% 0% 0% 75%)", x: "32vw", y: "20vh", scale: 0.8, blur: "7px", initialOpacity: 0 }
  ];

  // Always run the full 2.5s boot loader on every visit (no session skip).
  const handleBootComplete = () => {
    setBooted(true);
  };

  // GSAP Cinematic Transitions & Pinning (Brutalist Aesthetic)
  useGSAP(() => {
    if (!booted) return;

    // 1. Entrance animation timeline (starts immediately on mount, which is after loader unmounts)
    const tl = gsap.timeline();

    tl.fromTo(".hero-tag-animate", 
      { opacity: 0, y: 15 },
      { opacity: 1, y: 0, duration: 0.8, stagger: 0.1, ease: "power3.out" }
    );

    tl.fromTo(".hero-split-word",
      { y: "100%", opacity: 0 },
      { y: "0%", opacity: 1, duration: 1.2, stagger: 0.15, ease: "power4.out" },
      "-=0.6"
    );

    tl.fromTo(".hero-sub-line",
      { scaleX: 0 },
      { scaleX: 1, duration: 1, transformOrigin: "left center", ease: "power3.out" },
      "-=0.8"
    );

    tl.fromTo(".hero-sub-text",
      { opacity: 0, x: -10 },
      { opacity: 1, x: 0, duration: 0.8, ease: "power3.out" },
      "-=0.6"
    );

    tl.fromTo(".hero-btn-animate",
      { opacity: 0, y: 20 },
      { opacity: 1, y: 0, duration: 0.8, stagger: 0.15, ease: "power3.out" },
      "-=0.6"
    );

    // 2. ScrollTrigger animations for sections
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

    // 3. Image Assembly Scroll Animation (Produx parity sequence)
    const assemblyTl = gsap.timeline({
      scrollTrigger: {
        trigger: "#assembly-sticky-trigger",
        start: "top top",
        end: "+=400%", // Pinned for a massive, gradual 10-scroll feel!
        pin: true,
        scrub: 1.5,
      }
    });

    // Text box stays still for a bit, then fades out and moves up (duration: 1.2)
    assemblyTl.to(".assembly-text-box", {
      y: "-120px",
      opacity: 0,
      filter: "blur(6px)",
      duration: 1.2,
      ease: "power2.inOut",
    }, 0.2); // starts at 0.2, ends at 1.4

    // 12 Slices slide in from scattered positions and merge (duration: 2.2)
    pieces.forEach((p) => {
      assemblyTl.fromTo(`.assembly-slice-${p.id}`,
        { x: p.x, y: p.y, scale: p.scale, opacity: p.initialOpacity, filter: `blur(${p.blur})` },
        { x: "0vw", y: "0vh", scale: 1, opacity: 1, filter: "blur(0px)", ease: "power2.inOut", duration: 2.2 },
        0.6 // starts at 0.6, ends at 2.8
      );
    });

    // Fade in the dark gradient overlay once slices assemble
    assemblyTl.fromTo(".assembly-scrim",
      { opacity: 0 },
      { opacity: 1, duration: 1.0, ease: "power2.inOut" },
      2.0 // starts at 2.0, ends at 3.0
    );

    // Manifesto text overlays fade in
    assemblyTl.fromTo(".assembly-text-overlay",
      { opacity: 0, y: 20 },
      { opacity: 1, y: 0, duration: 0.8, ease: "power3.out", stagger: 0.2 },
      2.6 // starts at 2.6, ends at 3.4
    );

    // Zoom out at the end for clean exit transition
    assemblyTl.to(".assembly-image-container", {
      scale: 0.95,
      opacity: 0.9,
      duration: 0.6,
      ease: "power2.inOut",
    }, 3.4); // starts at 3.4, ends at 4.0
  }, { dependencies: [booted], scope: containerRef });

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
            {/* Redesigned Pinned Second Section with Image Assembly */}
            <section id="assembly-sticky-trigger" className="relative w-full h-screen overflow-hidden flex flex-col justify-center items-center bg-transparent z-10">
              
              {/* Left: Heading and CTAs (positioned absolute to float over background) */}
              <div className="assembly-text-box absolute left-[5.5vw] top-[16vh] flex flex-col items-start text-left z-20 pointer-events-auto max-lg:relative max-lg:left-0 max-lg:top-0 max-lg:px-[4.10vw] max-sm:px-[5.97vw] max-lg:mb-[4vh] w-full max-w-xl">
                <div className="flex items-center gap-3 mb-6 flex-wrap">
                  <span className="nb-tag hero-tag-animate"><span className="text-[var(--red-700)]">◆</span> casper · testnet live</span>
                  <span className="nb-tag nb-tag-ghost hero-tag-animate">/// vol.𝟎𝟏 — agent economy</span>
                  <span className="nb-index hero-tag-animate">𝟐𝟎𝟐𝟔</span>
                </div>
                <h1 className="mb-1 leading-none text-white uppercase tracking-tight" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, fontSize: "clamp(36px, 5.5vw, 64px)" }}>
                  Agentic
                </h1>
                <h1 className="mb-6 leading-none text-white uppercase tracking-tight" style={{ fontFamily: "var(--font-tech), 'Sora', sans-serif", fontWeight: 300, fontSize: "clamp(36px, 5.5vw, 64px)" }}>
                  Infrastructure
                </h1>
                <div className="flex items-center gap-4 mb-8 w-full">
                  <div className="tech-line flex-1 hero-sub-line origin-left" />
                  <span className="nb-index whitespace-nowrap hero-sub-text">escrow · oracle · the tower</span>
                </div>
                <div className="flex gap-4 flex-wrap">
                  <button className="button-primary inline-flex items-center hero-btn-animate">Deploy Now<span className="btn-icon-circle ml-2">↗</span></button>
                  <button className="btn-ghost hero-btn-animate">Talk to Sales</button>
                </div>
              </div>

              {/* Center/Bottom: Image Assembly Box */}
              <div className="assembly-image-container relative w-[75vw] max-w-[1200px] aspect-[1.784/1] z-10 pointer-events-auto mt-[20vh] max-lg:w-[90vw] max-lg:max-w-none">
                
                {/* 12 Slices of image (NO bounding boxes, NO borders, floating in open space!) */}
                {pieces.map((p) => (
                  <div 
                    key={p.id}
                    className={`absolute inset-0 assembly-slice assembly-slice-${p.id}`} 
                    style={{ 
                      backgroundImage: "url(/anime_robot.jpeg)",
                      backgroundSize: "cover",
                      backgroundPosition: "center",
                      clipPath: p.inset,
                      opacity: p.initialOpacity,
                    }} 
                  />
                ))}

                {/* Clean gradient overlay on the image instead of solid dark scrim */}
                <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-black/20 to-black/40 z-10 pointer-events-none assembly-scrim" style={{ opacity: 0 }} />

                {/* Integrated Manifesto Text Overlaid on Image */}
                <div 
                  className="absolute left-[8%] bottom-[12%] z-20 flex flex-col items-start text-left assembly-text-overlay font-mono lowercase tracking-[0.14em] opacity-0"
                  style={{ textShadow: "0 2px 8px rgba(0,0,0,0.9)" }}
                >
                  <p className="label-18 text-white mb-2"><span className="text-[var(--red-700)]">◆</span> for coding agents</p>
                  <p className="label-14 text-white/80">↳ to ship apps and agents</p>
                  <p className="label-14 text-white/80">↳ automated by agents</p>
                </div>

                <div 
                  className="absolute right-[8%] bottom-[12%] z-20 assembly-text-overlay font-mono lowercase tracking-[0.14em] text-right opacity-0"
                  style={{ textShadow: "0 2px 8px rgba(0,0,0,0.9)" }}
                >
                  <p className="label-14 text-[var(--red-700)] font-bold">settled on casper · live on-chain</p>
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
