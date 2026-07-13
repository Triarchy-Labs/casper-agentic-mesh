"use client";
import React, { useRef, useState, useEffect } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { CustomEase } from "gsap/CustomEase";
import { ScrollHero } from "@/components/ScrollHero";
import { ManifestoReveal } from "@/components/ManifestoReveal";
import { PixelDecodeText } from "@/components/PixelDecodeText";
import { EcosystemStrips } from "@/components/EcosystemStrips";
import { CrystalForge } from "@/components/CrystalForge";
import { AnimatePresence, motion, useScroll, useMotionValueEvent } from "framer-motion";
import BootSequence from "@/components/BootSequence";
import { CornerMarks } from "@/components/AgentNetworkGrid";
import { CarbonFabric } from "@/components/CarbonFabric";
import { CinematicDim } from "@/components/CinematicDim";

gsap.registerPlugin(ScrollTrigger, CustomEase);

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
    if (!booted) return;



    // 2. ScrollTrigger animations for sections
    CustomEase.create("natureSway", "M0,0 C0.08,0.494 0.14,1 1,1");
    CustomEase.create("premiumGlide", "M0,0 C0.1,0.4 0.15,1 1,1"); // from the produx bundle

    // Card entrance — produx's real technique, applied to the RIGHT element this time. In their
    // bundle the clip-wipe targets the WHOLE visible block (hero canvas / whole image container):
    // before it fires there is only page void, then the block "stretches out of nothing" — clip
    // inset(100% 100% 0 0) -> 0 WHILE scale 1.3 -> 1 from "left bottom", 1.7s power4.out (their
    // hero-canvas pairing). Clipping only the photo INSIDE an already-visible framed panel (my
    // earlier attempt) produced the black-hole glitch. So: wipe the ENTIRE panel, done in the
    // live-rect scroll handler below (ScrollTrigger start/end race the pin spacers here).

    // Header tag blur-fade; the h2 itself is the canvas pixel-decode (PixelDecodeText playOnEnter),
    // which self-triggers at top 65% with produx's 1.75s sine.out — the decompressor beat.
    gsap.fromTo(".synergy-header .nb-tag",
      { opacity: 0, y: 14, filter: "blur(6px)" },
      {
        opacity: 1, y: 0, filter: "blur(0px)", duration: 1, ease: "natureSway", delay: 0.1,
        scrollTrigger: { trigger: ".synergy-header", start: "top 95%", toggleActions: "play none none reverse" },
      }
    );




    // 4. Produx card-image mechanics — 1:1 from their bundle, no improvisation:
    //    - scale(1.15) lives INLINE in the markup (like their wrapper), so no tween-cache accident
    //      can drop it (a lost scale = zero headroom = the black bars that appeared).
    //    - clip-path ENTRANCE: image starts inset(100% 100% 0% 0%) and wipes open to inset(0) over
    //      1.7s power4.out at "top 85%", set open instantly if already in view (their exact code).
    //    - parallax: ScrollTrigger.create + onUpdate set(yPercent, mapRange(0,1,-10,10, progress)),
    //      trigger the frame, "top bottom" -> "bottom top", scrub: true. gsap never touches scale.
    // Driven off the LIVE getBoundingClientRect instead of precomputed trigger positions.
    // Measured live: ScrollTrigger was failing to add the hero pin-spacer offset (4752px) to these
    // triggers across gsap contexts (cards start=552 vs real doc top 5304 — while the crystal pin
    // DID get the offset), so their progress sat at 1 and the parallax froze at yPercent +10 = the
    // persistent top bar. Live rects are immune to spacer/refresh races; the math is the same
    // produx formula: progress = (vh - top) / (vh + height) -> yPercent -10..+10.
    // produx caption reveal (their card "reveal" batch): title words roll from masks (natureSway
    // .9s stagger .1), description words fast variant (premiumGlide .65s), pointer square scales in
    // (natureSway .7s). Words split at runtime into the same mask structure as their nf.default.
    const splitWords = (el: HTMLElement | null) => {
      if (!el) return [] as HTMLElement[];
      const words = (el.textContent || "").trim().split(/\s+/);
      el.innerHTML = words.map((w) => `<span class="cwm"><span class="cw">${w}</span></span>`).join(" ");
      return Array.from(el.querySelectorAll<HTMLElement>(".cw"));
    };

    const photoState = gsap.utils.toArray<HTMLElement>(".card-photo").map((photo) => {
      const frame = (photo.closest(".synergy-section") as HTMLElement | null) ?? photo;
      // caption elements live in the same grid cell as the panel
      const cell = (frame.closest(".focus-cards > div") as HTMLElement | null) ?? frame.parentElement!;
      const pointer = cell.querySelector<HTMLElement>(".card-pointer");
      const titleWords = splitWords(cell.querySelector<HTMLElement>(".card-title"));
      const descWords = splitWords(cell.querySelector<HTMLElement>(".card-desc"));
      // wipe state: whole panel hidden (void) until it crosses top 85% — produx guard: if already
      // in view on arrival, it is simply open, captions visible, no animation.
      let wipe = true;
      if (frame.getBoundingClientRect().top < 0.85 * window.innerHeight) {
        wipe = false;
        gsap.set([...titleWords, ...descWords], { y: "0%", opacity: 1 });
      } else {
        // NB: no scale pairing — that's their hero-canvas variant only; on a grid a scaled panel
        // would bleed over neighbours. The "stretch" read comes from the inner art being
        // scale(1.15) + parallax-shifted while the container wipes open — which we have.
        gsap.set(frame, { clipPath: "inset(100% 100% 0% 0%)" });
        if (pointer) gsap.set(pointer, { scale: 0 });
      }
      return { photo, frame, wipe, pointer, titleWords, descWords };
    });
    const onCardScroll = () => {
      const vh = window.innerHeight;
      photoState.forEach((s) => {
        const r = s.frame.getBoundingClientRect();
        if (r.bottom < -60 || r.top > vh + 60) return; // offscreen — skip
        const p = Math.max(0, Math.min(1, (vh - r.top) / (vh + r.height)));
        gsap.set(s.photo, { yPercent: -10 + 20 * p });
        if (s.wipe && r.top < vh * 0.85) {
          s.wipe = false;
          gsap.to(s.frame, { clipPath: "inset(0% 0% 0% 0%)", duration: 1.7, ease: "power4.out" });
          // produx card "reveal" batch — their exact values
          if (s.pointer) gsap.to(s.pointer, { scale: 1, duration: 0.7, ease: "natureSway" });
          gsap.to(s.titleWords, { y: "0%", opacity: 1, duration: 0.9, stagger: 0.1, ease: "natureSway" });
          gsap.to(s.descWords, { y: "0%", opacity: 1, duration: 0.65, stagger: 0.03, ease: "premiumGlide", delay: 0.1 });
        }
      });
    };
    window.addEventListener("scroll", onCardScroll, { passive: true });
    onCardScroll();

    // 5. Produx parity — focus grid. EXACT values traced from the minified card handler.
    // Hovered: opacity 1, scale 1.02, no blur. Siblings: opacity 0.5, scale 0.98,
    // filter blur(4px) brightness(0.4). duration 0.6, ease power2.out, overwrite true.
    const focusCards = gsap.utils.toArray<HTMLElement>(".focus-cards > div");
    // Pre-warm GPU layers: the first hover used to apply blur(4px) to 4 huge panels at once,
    // forcing first-time layer promotion + rasterization of giant textures = one long frame where
    // layers weren't ready ("the screen blinked black"). Setting a non-none filter + force3D at
    // mount creates those layers up-front during idle, so the first hover only animates them.
    gsap.set(focusCards, { filter: "blur(0px) brightness(1)", force3D: true, willChange: "transform, opacity, filter" });
    const focusCleanups: Array<() => void> = [];
    focusCards.forEach((card) => {
      const enter = () => {
        // produx sets sibling opacity 0.5, but their bg is near-black so it reads as pure dimming.
        // Our bg is the red carbon fabric, so opacity shows through — keep cards opaque and
        // dim via brightness+blur only (matches their look on our background).
        // produx dims siblings to ~50% luminance — but their photos are BRIGHT (white billboards),
        // clearly readable dimmed (verified against their live hover, user video). Our arts are
        // dark to begin with, so we crush less: brightness 0.75 on our palette lands at the same
        // "darker + blurred but the photo still reads" perception as their 0.5-ish.
        gsap.to(
          focusCards.filter((c) => c !== card),
          { opacity: 1, scale: 0.98, filter: "blur(4px) brightness(0.75)", duration: 0.6, ease: "power2.out", overwrite: true }
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

    // CRITICAL: these triggers race the hero/crystal pins — measured live, the card triggers were
    // computed in a layout WITHOUT the pin spacers (starts ~714-2792 while the hero pin alone spans
    // 0-3672), so past the hero they sat at progress=1 forever: parallax frozen at yPercent +10 =
    // the persistent top bar. A deferred global refresh re-measures everything with all pin
    // spacers registered. Second pass catches late image/layout shifts.
    gsap.delayedCall(0.3, () => ScrollTrigger.refresh());
    gsap.delayedCall(1.5, () => ScrollTrigger.refresh());

    return () => {
      window.removeEventListener("scroll", onCardScroll);
      focusCleanups.forEach((fn) => fn());
    };
  }, { dependencies: [booted], scope: containerRef });

  // The hero + mosaic pins insert pin-spacers that push the un-pinned #manifesto down the page.
  // ScrollTrigger measures the manifesto trigger before those spacers settle, so its start/end
  // land in the wrong place and the lines stay hidden. Force a recalc once everything is mounted.
  useEffect(() => {
    if (!booted) return;
    const ids = [120, 400, 900].map((d) => setTimeout(() => ScrollTrigger.refresh(), d));
    return () => ids.forEach(clearTimeout);
  }, [booted]);

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



            {/* Synergy Dashboard Cinematic Chapters.
                produx overlap (their hero has -mb-[17.5vh]): this block rides 17.5vh up onto the
                mosaic's tail — the overlap zone is top PADDING, so nothing covers the image, but the
                heading enters the frame right at pin release instead of after an empty runway. That
                empty runway was why the wheel got spun hard at the mosaic exit -> the jerk. */}
            <div ref={sectionsRef} className="w-full -mt-[17.5vh] px-[5.5vw] pt-[17vh] pb-[120px] flex flex-col gap-12">
              
              {/* Section header — the decompressor after the mosaic pin. The h2 is produx's exact
                  heading treatment ('Selected projects' in their bundle = the nP.default CANVAS
                  pixel-decode, time-based: 1.75s sine.out, fired once at top 65% — and being a
                  canvas it's not cursor-selectable, same as theirs). Red edge instead of lime. */}
              <div className="synergy-header flex flex-col gap-4 mb-2">
                <span className="nb-tag w-max"><span className="text-[var(--red-700)]">◆</span> the mesh · five vectors</span>
                <h2 className="max-w-[46vw] max-lg:max-w-none">
                  <PixelDecodeText text="Absolute Synergy" playOnEnter fontVw={0.06} fsMax={112} />
                </h2>
              </div>

              {/* Produx-style: 3 grid rows, flex-col gap-[13.67vh] between rows (from dump) */}
              <div className="flex flex-col gap-[13.67vh] max-sm:gap-[6vh] pt-[11.7vh] max-sm:pt-[4vh]">

                {/* Row 1: big-left (7) + small-right-down (4) */}
                <div className="focus-cards grid grid-cols-12 gap-x-[1.39vw] gap-y-[13.67vh] max-sm:grid-cols-1 max-sm:gap-y-[6vh]">
                  {/* 01 ESCROW — col-span-7, big left */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-7 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh] md:min-h-0 md:aspect-[1.27/1] group overflow-hidden">
                      <CornerMarks />
                      {/* Vector Illustration Background — aspect from produx Payy card (2211×1740) */}
                      <div
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.85] group-hover:opacity-100 transition-opacity duration-500 pointer-events-none z-0 will-change-transform"
                        style={{
                          backgroundImage: "url(/vector_escrow.jpeg)",
                          transform: "scale(1.15)",
                          transformOrigin: "center center",
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
                      <div className="card-pointer my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="card-title heading-32 leading-tight">Autonomous Escrow</h3>
                        <p className="card-desc label-13-mono text-[var(--gray-800)] uppercase">The zero-trust bedrock of the mesh. Rust/WASM smart contracts with real on-chain settlement.</p>
                      </div>
                    </div>
                  </div>

                  {/* 03 OMNI-MESH — col-span-4, right, pushed down (mt-auto) */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-4 md:col-end-13 md:mt-auto md:h-fit cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative group overflow-hidden min-h-[48.8vh] md:min-h-0 md:aspect-[1.04/1]">
                      <CornerMarks />
                      {/* Vector Illustration Background */}
                      <div 
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.85] group-hover:opacity-100 transition-opacity duration-500 pointer-events-none z-0 will-change-transform"
                        style={{ 
                          backgroundImage: "url(/card-omni-mesh.jpg)",
                          transform: "scale(1.15)",
                          transformOrigin: "center center",
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
                      <div className="card-pointer my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="card-title heading-32 leading-tight">The Omni-Mesh</h3>
                        <p className="card-desc label-13-mono text-[var(--gray-800)] uppercase">The economic OS for AI agents. All vectors converging into one organism.</p>
                      </div>
                    </div>
                  </div>

                  {/* Row 2: wide centered (10) — RWA Risk Oracle */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-10 md:col-start-2 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[3.5vw] md:p-[4.5vw] relative overflow-hidden border-[var(--red-900)] text-center min-h-[58vh] md:min-h-0 md:aspect-[1.9/1] flex flex-col justify-center group">
                      <CornerMarks />
                      {/* Vector Illustration Background */}
                      <div 
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.85] group-hover:opacity-100 transition-opacity duration-500 pointer-events-none z-0 will-change-transform"
                        style={{ 
                          backgroundImage: "url(/vector_oracle.jpeg)",
                          transform: "scale(1.15)",
                          transformOrigin: "center center",
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
                      <div className="card-pointer my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="card-title heading-32 leading-tight">RWA Risk Oracle</h3>
                        <p className="card-desc label-13-mono text-[var(--gray-800)] uppercase">The Sentinel. A live on-chain data feed with agent identity and accruing reputation.</p>
                      </div>
                    </div>
                  </div>

                  {/* Row 3: small-left (4) + big-right (7) */}
                  {/* 04 TRIBUNAL — col-span-4, small left */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-4 md:h-fit cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh] md:min-h-0 md:aspect-[0.96/1] overflow-hidden group">
                      <CornerMarks />
                      <div
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.85] group-hover:opacity-100 transition-opacity duration-500 pointer-events-none z-0 will-change-transform"
                        style={{
                          backgroundImage: "url(/card-tribunal.jpg)",
                          transform: "scale(1.15)",
                          transformOrigin: "center center",
                        }}
                      />
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
                      <div className="card-pointer my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="card-title heading-32 leading-tight">Agent Tribunal</h3>
                        <p className="card-desc label-13-mono text-[var(--gray-800)] uppercase">An adversarial court of real models that rules on work and moves CSPR on-chain.</p>
                      </div>
                    </div>
                  </div>

                  {/* 05 x402 PAYMENT — col-span-7, big right */}
                  <div className="flex flex-col gap-[1.67vw] col-span-12 md:col-span-7 md:col-end-13 cursor-pointer">
                    <div className="synergy-section editorial-panel p-[2.22vw] relative min-h-[48.8vh] md:min-h-0 md:aspect-[1.2/1] overflow-hidden group">
                      <CornerMarks />
                      <div
                        className="card-photo absolute inset-0 bg-cover bg-center opacity-[0.85] group-hover:opacity-100 transition-opacity duration-500 pointer-events-none z-0 will-change-transform"
                        style={{
                          backgroundImage: "url(/card-x402.jpg)",
                          transform: "scale(1.15)",
                          transformOrigin: "center center",
                        }}
                      />
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
                      <div className="card-pointer my-[1.5vh] mr-[0.73vw] size-[0.55vw] border border-[#303030] max-sm:hidden shrink-0"></div>
                      <div className="flex flex-col gap-[0.73vw]">
                        <h3 className="card-title heading-32 leading-tight">x402 Payment Layer</h3>
                        <p className="card-desc label-13-mono text-[var(--gray-800)] uppercase">HTTP 402 native. Agents pay per API call with real CSPR — no subscriptions, no keys, just value for value.</p>
                      </div>
                    </div>
                  </div>
                </div>

              </div>

            </div>

            {/* Thesis — tall sticky section so the pixel-decode plays over ~2-3 scrolls, text held in view */}
            <ManifestoReveal />

            {/* Ecosystem — angled Off-White strips: the buildathon context, forged in the open */}
            <section className="relative w-full px-[5.5vw] pb-[22vh] z-10">
              <EcosystemStrips />
            </section>

            {/* Crystal forge — produx rock section rebuilt: scroll-scrubbed crystal charging red,
                gradient headline, then the three verdict points pop in on the hero pose */}
            <CrystalForge />
          </div>
        </main>
      )}
    </>
  );
}
