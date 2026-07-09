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
  // Wider coordinates, custom start times, durations, and easings for asymmetrical, organic assembly
  const pieces = [
    // Row 1
    { id: 1, inset: "inset(0% 75% 66.66% 0%)", x: "-40vw", y: "-35vh", scale: 0.75, blur: "8px", initialOpacity: 0.8, startTime: 0.8, duration: 1.6, ease: "power3.out" },
    { id: 2, inset: "inset(0% 50% 66.66% 25%)", x: "-10vw", y: "-40vh", scale: 0.7, blur: "5px", initialOpacity: 0, startTime: 1.2, duration: 1.2, ease: "back.out(1.2)" },
    { id: 3, inset: "inset(0% 25% 66.66% 50%)", x: "15vw", y: "-38vh", scale: 0.85, blur: "9px", initialOpacity: 0.8, startTime: 0.9, duration: 1.8, ease: "power2.inOut" },
    { id: 4, inset: "inset(0% 0% 66.66% 75%)", x: "42vw", y: "-30vh", scale: 0.8, blur: "6px", initialOpacity: 0, startTime: 1.4, duration: 1.0, ease: "sine.out" },
    // Row 2
    { id: 5, inset: "inset(33.33% 75% 33.33% 0%)", x: "-45vw", y: "-10vh", scale: 0.7, blur: "8px", initialOpacity: 0.8, startTime: 0.7, duration: 2.0, ease: "power4.out" },
    { id: 6, inset: "inset(33.33% 50% 33.33% 25%)", x: "-20vw", y: "20vh", scale: 0.75, blur: "6px", initialOpacity: 0, startTime: 1.3, duration: 1.4, ease: "power2.out" },
    { id: 7, inset: "inset(33.33% 25% 33.33% 50%)", x: "25vw", y: "-15vh", scale: 0.85, blur: "7px", initialOpacity: 0.8, startTime: 1.0, duration: 1.5, ease: "back.out(1.5)" },
    { id: 8, inset: "inset(33.33% 0% 33.33% 75%)", x: "45vw", y: "10vh", scale: 0.75, blur: "5px", initialOpacity: 0, startTime: 1.5, duration: 1.1, ease: "power3.inOut" },
    // Row 3
    { id: 9, inset: "inset(66.66% 75% 0% 0%)", x: "-38vw", y: "30vh", scale: 0.8, blur: "7px", initialOpacity: 0.8, startTime: 0.8, duration: 1.7, ease: "power2.out" },
    { id: 10, inset: "inset(66.66% 50% 0% 25%)", x: "-15vw", y: "40vh", scale: 0.65, blur: "9px", initialOpacity: 0, startTime: 1.1, duration: 1.3, ease: "sine.inOut" },
    { id: 11, inset: "inset(66.66% 25% 0% 50%)", x: "15vw", y: "38vh", scale: 0.7, blur: "6px", initialOpacity: 0.8, startTime: 1.2, duration: 1.6, ease: "power3.out" },
    { id: 12, inset: "inset(66.66% 0% 0% 75%)", x: "40vw", y: "35vh", scale: 0.75, blur: "8px", initialOpacity: 0, startTime: 0.6, duration: 2.2, ease: "power4.inOut" }
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
        end: "+=450%", // Pinned for a massive, gradual 12-scroll feel!
        pin: true,
        scrub: 1.5,
      }
    });

    // Text box exit starts at 2.0, ends at 2.8
    assemblyTl.to(".assembly-text-box", {
      opacity: 0,
      y: "-100px",
      filter: "blur(6px)",
      duration: 0.8,
      ease: "power2.in",
    }, 2.0);

    // 12 Slices fade in from opacity 0 to initialOpacity (0.0 to 0.8) and then merge with custom non-linear parameters
    pieces.forEach((p) => {
      assemblyTl.fromTo(`.assembly-slice-${p.id}`,
        { x: p.x, y: p.y, scale: p.scale, opacity: 0, filter: `blur(${p.blur})` },
        { opacity: p.initialOpacity, duration: 0.8, ease: "power1.inOut" },
        0.0
      );
      assemblyTl.to(`.assembly-slice-${p.id}`,
        { x: "0vw", y: "0vh", scale: 1, opacity: 1, filter: "blur(0px)", ease: p.ease, duration: p.duration },
        p.startTime
      );
    });

    // Fade in the dark gradient overlay once slices assemble
    assemblyTl.fromTo(".assembly-scrim",
      { opacity: 0 },
      { opacity: 1, duration: 0.8, ease: "power2.inOut" },
      1.8 // starts at 1.8, ends at 2.6
    );

    // Manifesto text overlays fade in
    assemblyTl.fromTo(".assembly-text-overlay",
      { opacity: 0, y: 20 },
      { opacity: 1, y: 0, duration: 0.8, ease: "power3.out", stagger: 0.2 },
      2.0 // starts at 2.0, ends at 2.8
    );

    // Image container exit (2.8 to 3.4) - disappears completely ~2 scrolls after text
    assemblyTl.to(".assembly-image-container", {
      y: "-120px",
      opacity: 0,
      scale: 0.95,
      filter: "blur(8px)",
      duration: 0.6,
      ease: "power2.in",
    }, 2.8); // starts at 2.8, ends at 3.4

    // 4. Produx parity — photo parallax inside each card frame.
    // Image pre-scaled 1.15 for headroom, drifts yPercent on scroll with scrub 1.5 (dump values).
    const photos = gsap.utils.toArray<HTMLElement>(".card-photo");
    photos.forEach((photo) => {
      gsap.set(photo, { scale: 1.15, transformOrigin: "center center" });
      const frame = photo.closest(".synergy-section") as HTMLElement | null;
      // Dump: yPercent magnitude 10, ease none, start "top bottom" → end "bottom top".
      gsap.fromTo(
        photo,
        { yPercent: -5 },
        {
          yPercent: 5,
          ease: "none",
          scrollTrigger: {
            trigger: frame ?? photo,
            start: "top bottom",
            end: "bottom top",
            scrub: 1.5,
          },
        }
      );
    });

    // 5. Produx parity — focus grid. EXACT values traced from the minified card handler.
    // Hovered: opacity 1, scale 1.02, no blur. Siblings: opacity 0.5, scale 0.98,
    // filter blur(4px) brightness(0.4). duration 0.6, ease power2.out, overwrite true.
    const focusCards = gsap.utils.toArray<HTMLElement>(".focus-cards > div");
    const focusCleanups: Array<() => void> = [];
    focusCards.forEach((card) => {
      const enter = () => {
        gsap.to(
          focusCards.filter((c) => c !== card),
          { opacity: 0.5, scale: 0.98, filter: "blur(4px) brightness(0.4)", duration: 0.6, ease: "power2.out", overwrite: true }
        );
        gsap.to(card, { opacity: 1, scale: 1.02, filter: "blur(0px) brightness(1)", duration: 0.6, ease: "power2.out", overwrite: true });
      };
      const leave = () => {
        gsap.to(focusCards, { opacity: 1, scale: 1, filter: "blur(0px) brightness(1)", duration: 0.6, ease: "power2.out", overwrite: true });
      };
      card.addEventListener("mouseenter", enter);
      card.addEventListener("mouseleave", leave);
      focusCleanups.push(() => {
        card.removeEventListener("mouseenter", enter);
        card.removeEventListener("mouseleave", leave);
      });
    });

    return () => {
      focusCleanups.forEach((fn) => fn());
    };
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
              <div className="assembly-image-container relative w-[80vw] max-w-[1600px] aspect-[1.784/1] z-10 pointer-events-auto mt-[10vh] max-lg:w-[90vw] max-lg:max-w-none" style={{ maskImage: "radial-gradient(ellipse, black 40%, transparent 92%)", WebkitMaskImage: "radial-gradient(ellipse, black 40%, transparent 92%)" }}>
                
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
                      opacity: 0,
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
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh] md:min-h-0 md:aspect-[1.27/1] group overflow-hidden">
                      <CornerMarks />
                      {/* Vector Illustration Background — aspect from produx Payy card (2211×1740) */}
                      <div
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.40] group-hover:opacity-[0.95] transition-opacity duration-500 pointer-events-none mix-blend-screen z-0 will-change-transform"
                        style={{
                          backgroundImage: "url(/vector_escrow.jpeg)",
                          maskImage: "radial-gradient(ellipse at center, black 40%, transparent 100%)",
                          WebkitMaskImage: "radial-gradient(ellipse at center, black 40%, transparent 100%)"
                        }}
                      />
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

                  {/* 03 OMNI-MESH — col-span-4, right, pushed down (mt-auto) */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-4 md:col-end-13 md:mt-auto md:h-fit cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative group overflow-hidden min-h-[48.8vh] md:min-h-0 md:aspect-[1.04/1]">
                      <CornerMarks />
                      {/* Vector Illustration Background */}
                      <div 
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.40] group-hover:opacity-[0.95] transition-opacity duration-500 pointer-events-none mix-blend-screen z-0 will-change-transform"
                        style={{ 
                          backgroundImage: "url(/anime_robot.jpeg)",
                          maskImage: "radial-gradient(ellipse at center, black 40%, transparent 100%)",
                          WebkitMaskImage: "radial-gradient(ellipse at center, black 40%, transparent 100%)"
                        }}
                      />
                      <p className="label-14-mono text-[var(--red-700)] mb-[16px] z-10">03 // vector gamma</p>
                      <ul className="label-13-mono text-[var(--gray-800)] space-y-[12px] flex flex-col z-10 mt-auto">
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> escrow · oracle · tribunal</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> the tower overseer</li>
                        <li className="flex items-center gap-[8px]"><span className="text-[var(--red-700)]">→</span> proof of liveness</li>
                      </ul>
                      <div className="panel-cta z-10">
                        <span className="panel-cta-square" />
                        <span className="panel-cta-text">initialize synergy</span>
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

                  {/* Row 2: wide centered (10) — RWA Risk Oracle */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-10 md:col-start-2 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[3.5vw] md:p-[4.5vw] relative overflow-hidden border-[var(--red-900)] text-center min-h-[58vh] md:min-h-0 md:aspect-[1.9/1] flex flex-col justify-center group">
                      <CornerMarks />
                      {/* Vector Illustration Background */}
                      <div 
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.40] group-hover:opacity-[0.95] transition-opacity duration-500 pointer-events-none mix-blend-screen z-0 will-change-transform"
                        style={{ 
                          backgroundImage: "url(/vector_oracle.jpeg)",
                          maskImage: "radial-gradient(ellipse at center, black 40%, transparent 100%)",
                          WebkitMaskImage: "radial-gradient(ellipse at center, black 40%, transparent 100%)"
                        }}
                      />
                      <div className="relative z-10 flex flex-col items-center">
                        <span className="nb-tag mb-6"><span className="text-[var(--red-700)]">◆</span> 02 · RWA Risk Oracle · Vector Beta</span>
                        <h2 className="nb-display text-[clamp(44px,7vw,96px)] mb-[24px]">RWA Risk Oracle</h2>
                        <p className="copy-18 text-[var(--gray-900)] mb-[36px] max-w-2xl">
                          The Sentinel. A live on-chain data feed with agent identity and accruing reputation, settled on Casper.
                        </p>
                        <button className="button-primary">QUERY ORACLE</button>
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

                  {/* Row 3: small-left (4) + big-right (7) */}
                  {/* 04 TRIBUNAL — col-span-4, small left */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-4 md:h-fit cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh] md:min-h-0 md:aspect-[0.96/1] overflow-hidden">
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
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh] md:min-h-0 md:aspect-[1.2/1] overflow-hidden">
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
