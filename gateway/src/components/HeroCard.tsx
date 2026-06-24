// Hero card + scene, split into two exports:
//
//   <HeroScene />  — the ambient backdrop. Absolute-positioned
//                    starfield + conic holographic wash + halo.
//                    Drop it as the first child of a positioned
//                    hero <section> so it paints across the whole
//                    section.
//
//   <HeroCard />   — the actual virtual card in perspective space.
//                    Flows with normal layout, so it can sit in the
//                    right column of a hero grid on desktop and wrap
//                    below the text on tablet/mobile.
//
// Pointer tracking sets CSS custom properties on document.documentElement
// so both the scene and the card share state without needing a React
// context. A single useEffect registered once per page installs the
// listeners; it's a no-op if neither component ever mounts.
//
// The visual language (layered gradients, chip, sheen, grid texture,
// noise, orbs) is a faithful port of the standalone prototype at
// ~/code/cards402animation/index.html with three deltas:
//   - Idle drift dialled back from ±8/±6 to ±2.5/±1.8 per pass.
//   - The brand mark is replaced with a Cards402 wordmark rendered
//     via mask-image so it inherits the card's cream ink.
//   - The bottomline shows 'YOUR AGENT' instead of 'ASH / PRIMARY'.

'use client';

import { useEffect, useRef } from 'react';

// Shared init guard — if multiple HeroCard/HeroScene instances mount on
// the same page we only want one pointer listener, one rAF loop. The
// flag is module-scoped so the React strict-mode double-mount doesn't
// double up the listeners either.
let tiltInstalled = false;

function installTilt() {
  if (tiltInstalled || typeof window === 'undefined') return;
  tiltInstalled = true;

  const root = document.documentElement;

  // Kick the load-in progress variable to 1 on the first frame. Until
  // this runs, elements that multiply their opacity by
  // var(--load-progress) are fully hidden — so the sheen, grid texture,
  // noise speckle, orb highlights and corner rings all gate-in behind
  // the outline + shell intro instead of being visible from frame 1.
  requestAnimationFrame(() => root.style.setProperty('--load-progress', '1'));

  // Note: the tilt / parallax loop does NOT gate on
  // prefers-reduced-motion. An earlier commit did, but Windows
  // users who toggle "Show animations in Windows" off (a common
  // setting for perceived perf) end up with prefers-reduced-motion:
  // reduce set, and they didn't expect parallax to disappear as a
  // side effect. The real perf cost was never the pointer tilt loop
  // — it was the filter: blur(...) on pointer-bound backgrounds
  // (scene, halo, card-shadow) and the continuous hc-floatCard filter
  // animation. Those were removed in 4845dcc, so this rAF is cheap
  // enough to run unconditionally: six CSS variable writes per
  // frame driving composite-only transforms.

  // Tilt targets (pixels / degrees) + lerped current values
  let pointerX = 0;
  let pointerY = 0;
  let currentX = 0;
  let currentY = 0;
  // Sheen-glare pointer position in percent.
  let targetGlareX = 50;
  let targetGlareY = 50;
  let currentGlareX = 50;
  let currentGlareY = 50;
  let lastMove = performance.now();

  function setVar(name: string, value: string) {
    root.style.setProperty(name, value);
  }

  function onMove(e: PointerEvent) {
    // Read the root rect fresh on each event. An earlier refactor
    // tried to cache the rect and invalidate on scroll/resize — but
    // documentElement's rect changes on scroll (its top becomes
    // negative) and on any mid-page reflow, and the cache got stale
    // enough to break the parallax math. getBoundingClientRect on
    // documentElement is a few microseconds in practice — not worth
    // the correctness risk.
    const rect = root.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;
    const clampX = Math.max(0, Math.min(1, x));
    const clampY = Math.max(0, Math.min(1, y));
    pointerX = (clampX - 0.5) * 30;
    pointerY = (clampY - 0.5) * 22;
    targetGlareX = clampX * 100;
    targetGlareY = clampY * 100;
    lastMove = performance.now();
  }
  function onLeave() {
    pointerX = 0;
    pointerY = 0;
    targetGlareX = 50;
    targetGlareY = 50;
  }

  function animate() {
    // Dialled-down idle drift — original prototype was ±8/±6, too
    // busy for a financial surface. At ±2.5/±1.8 the card has a
    // gentle breath without bobbing.
    const idle = (performance.now() - lastMove) / 1000;
    const driftX = Math.sin(idle * 0.7) * 2.5;
    const driftY = Math.cos(idle * 0.6) * 1.8;
    const targetX = pointerX * 0.85 + driftX;
    const targetY = pointerY * 0.85 + driftY;

    currentX += (targetX - currentX) * 0.08;
    currentY += (targetY - currentY) * 0.08;
    currentGlareX += (targetGlareX - currentGlareX) * 0.08;
    currentGlareY += (targetGlareY - currentGlareY) * 0.08;

    setVar('--card-x', `${currentX}px`);
    setVar('--card-y', `${currentY}px`);
    setVar('--rotate-y', `${currentX * 0.55}deg`);
    setVar('--rotate-x', `${-currentY * 0.7}deg`);
    setVar('--pointer-x', `${currentGlareX}%`);
    setVar('--pointer-y', `${currentGlareY}%`);

    requestAnimationFrame(animate);
  }

  window.addEventListener('pointermove', onMove, { passive: true });
  // `pointerleave` on window is unreliable — use `pointerout` with a
  // null relatedTarget (which means the cursor left the viewport) as
  // a more consistent signal across browsers.
  window.addEventListener('pointerout', (e: PointerEvent) => {
    if (!e.relatedTarget) onLeave();
  });
  window.addEventListener('blur', onLeave);
  requestAnimationFrame(animate);
}

function useTilt() {
  useEffect(() => {
    installTilt();
  }, []);
}

// ─────────────────────────────────────────────────────────────────────
// HeroScene — absolute backdrop (starfield + conic glow + halo)
// ─────────────────────────────────────────────────────────────────────

export function HeroScene() {
  useTilt();
  return (
    <>
      <div className="hc-scene" aria-hidden>
        <div className="hc-halo" />
      </div>
      <SceneStyles />
    </>
  );
}

// ─────────────────────────────────────────────────────────────────────
// HeroCard — the card itself, flows with layout
// ─────────────────────────────────────────────────────────────────────

export function HeroCard() {
  const cardRef = useRef<HTMLElement>(null);
  useTilt();

  return (
    <>
      <div className="hc-card-wrap">
        <div className="hc-card-shadow" aria-hidden />
        <article ref={cardRef} className="hc-card" aria-label="Cards402 virtual card">
          {/* Load-in choreography elements (rendered first so they sit
              underneath the noise + content layers in z-order):
                · outline-glow: soft radial that pulses outward
                · card-outline: SVG rect that draws around the perimeter
                · card-shell:   holographic gradient that wipes upward
              See the keyframes block in CardStyles for the timing. */}
          <div className="hc-outline-glow" aria-hidden />
          <svg
            className="hc-card-outline"
            viewBox="0 0 384 600"
            preserveAspectRatio="none"
            aria-hidden
          >
            <rect x="1.5" y="1.5" width="381" height="597" rx="28" ry="28" />
          </svg>
          <div className="hc-card-shell" aria-hidden />
          <div className="hc-noise" aria-hidden />
          <div className="hc-rings" aria-hidden />
          <div className="hc-orb hc-orb-one" aria-hidden />
          <div className="hc-orb hc-orb-two" aria-hidden />
          <div className="hc-card-content">
            <header className="hc-topline">
              <span className="hc-brand-wordmark" role="img" aria-label="Cards402" />
              <div className="hc-topline-right">Virtual Card</div>
            </header>
            <div className="hc-middle">
              <div className="hc-chip" aria-hidden />
              <div className="hc-balance">
                <span>Available</span>
                <strong>$250.00</strong>
              </div>
              <div className="hc-digits">
                <div className="hc-label">Card Sequence</div>
                <div className="hc-value">4242 0402 4020 7890</div>
              </div>
            </div>
            <footer className="hc-bottomline">
              <div>YOUR AGENT</div>
              <div>VALID 12/28</div>
            </footer>
          </div>
        </article>
      </div>
      <CardStyles />
    </>
  );
}

// ─────────────────────────────────────────────────────────────────────
// Styles — split so each component carries what it needs. The scene
// owns the CSS variable initial-state defaults so it's valid to render
// <HeroScene> even if <HeroCard> is on the same page.
// ─────────────────────────────────────────────────────────────────────

function SceneStyles() {
  return (
    <style>{`
      :root {
        --card-x: 0px;
        --card-y: 0px;
        --rotate-x: -8deg;
        --rotate-y: 12deg;
        --pointer-x: 50%;
        --pointer-y: 50%;
        /* Gates the sheen + noise + orbs + rings opacity until the
           outline + shell intro has established the card shape.
           installTilt() flips this to 1 on the first rAF tick. */
        --load-progress: 0;
      }
      .hc-scene {
        position: absolute;
        inset: 0;
        isolation: isolate;
        overflow: hidden;
        pointer-events: none;
        background:
          radial-gradient(circle at 22% 22%, rgba(124, 245, 208, 0.09), transparent 26rem),
          radial-gradient(circle at 78% 18%, rgba(255, 125, 182, 0.07), transparent 22rem),
          radial-gradient(circle at 70% 85%, rgba(155, 209, 255, 0.08), transparent 30rem);
      }
      .hc-scene::before,
      .hc-scene::after {
        content: '';
        position: absolute;
        inset: -10%;
        pointer-events: none;
      }
      .hc-scene::before {
        background:
          radial-gradient(circle at 30% 35%, rgba(255, 255, 255, 0.08) 0 2px, transparent 3px),
          radial-gradient(circle at 65% 50%, rgba(255, 255, 255, 0.06) 0 1px, transparent 2px),
          radial-gradient(circle at 82% 28%, rgba(255, 255, 255, 0.08) 0 1px, transparent 2px),
          radial-gradient(circle at 18% 72%, rgba(255, 255, 255, 0.05) 0 1px, transparent 2px),
          radial-gradient(circle at 52% 82%, rgba(255, 255, 255, 0.08) 0 1px, transparent 2px);
        background-size: 320px 320px;
        opacity: 0.55;
        animation: hc-driftStars 32s linear infinite;
      }
      .hc-scene::after {
        /* Previously a conic gradient with filter: blur(90px). That blur
           was the single biggest perf cost on the homepage: when software
           rendering is in use (GPU compositing disabled, common on
           corporate Windows installs) the blur has to re-rasterize the
           entire hero region on every frame, and the transform binding
           on var(--card-x/y) meant every pointer move triggered that
           re-rasterize. Replaced with a radial wash that composites as
           a single background paint — no filter, no transform binding.
           Visual difference is small: still a soft multi-colour glow
           behind the card, just without the conic sweep. */
        background:
          radial-gradient(circle at 30% 30%, rgba(255, 207, 110, 0.08), transparent 55%),
          radial-gradient(circle at 72% 40%, rgba(124, 245, 208, 0.07), transparent 58%),
          radial-gradient(circle at 55% 80%, rgba(155, 209, 255, 0.08), transparent 60%),
          radial-gradient(circle at 85% 70%, rgba(255, 125, 182, 0.06), transparent 45%);
        opacity: 0.7;
      }

      .hc-halo {
        position: absolute;
        top: 50%;
        left: 72%;
        width: min(55vw, 36rem);
        aspect-ratio: 1;
        border-radius: 999px;
        pointer-events: none;
        /* Softer radial stop gives the halo look without needing
           filter: blur — the 58% transparent cutoff already produces
           a diffuse edge. The previous version layered a 14px blur
           on top AND re-rendered that blur on every pointer move
           because of the var(--card-x/y) transform binding. Now the
           halo is static. */
        background: radial-gradient(circle, rgba(255, 207, 110, 0.14), transparent 62%);
        opacity: 0.85;
        transform: translate(-50%, -50%);
      }
      .hc-halo::before,
      .hc-halo::after {
        content: '';
        position: absolute;
        inset: 18%;
        border-radius: 999px;
        border: 1px solid rgba(255, 255, 255, 0.07);
        pointer-events: none;
      }
      .hc-halo::after {
        inset: 30%;
        border-color: rgba(124, 245, 208, 0.12);
      }

      @media (max-width: 1100px) {
        .hc-halo {
          left: 62%;
        }
      }
      @media (max-width: 860px) {
        .hc-halo {
          left: 50%;
          top: 72%;
          width: min(85vw, 28rem);
        }
      }

      @keyframes hc-driftStars {
        from { transform: translate3d(0, 0, 0); }
        to { transform: translate3d(-120px, 70px, 0); }
      }
    `}</style>
  );
}

function CardStyles() {
  return (
    <style>{`
      .hc-card-wrap {
        position: relative;
        width: 100%;
        display: flex;
        justify-content: center;
        align-items: center;
        perspective: 1800px;
        transform-style: preserve-3d;
        /* Intro: the whole wrap lifts from 2rem below with a blur +
           scale, then settles. 1.4s ease-out-expo so it feels like
           the card is arriving, not just fading in. */
        animation: hc-wrapEnter 1400ms cubic-bezier(0.22, 1, 0.36, 1) both;
      }
      .hc-card-shadow {
        position: absolute;
        left: 10%;
        right: 10%;
        bottom: -2rem;
        height: 3rem;
        border-radius: 50%;
        /* Radial gradient alone produces a soft shadow without needing
           filter blur. The previous version had filter blur 20px plus
           a transform binding on var(--card-x/y), which forced a
           software-mode re-rasterize of the blurred region on every
           pointer move. Dropping both makes the shadow static but
           identical-looking. */
        background: radial-gradient(
          ellipse at center,
          rgba(255, 180, 70, 0.28),
          rgba(255, 180, 70, 0.08) 48%,
          rgba(0, 0, 0, 0) 72%
        );
        transform: translate3d(0, 0, -80px) scale(1.1);
        opacity: 0.8;
        pointer-events: none;
      }
      .hc-card {
        position: relative;
        width: min(84vw, 22rem);
        aspect-ratio: 0.64;
        border-radius: 1.7rem;
        overflow: hidden;
        background:
          linear-gradient(
            135deg,
            rgba(255, 255, 255, 0.14),
            rgba(255, 255, 255, 0.02) 35%,
            rgba(255, 255, 255, 0.08)
          ),
          linear-gradient(
            160deg,
            rgba(255, 207, 110, 0.16),
            rgba(124, 245, 208, 0.06) 42%,
            rgba(155, 209, 255, 0.16) 78%,
            rgba(255, 125, 182, 0.1)
          ),
          #0a0a0a;
        border: 1px solid rgba(255, 255, 255, 0.16);
        box-shadow:
          0 2rem 4rem rgba(0, 0, 0, 0.65),
          0 0 0 1px rgba(255, 255, 255, 0.04) inset,
          0 0 4rem rgba(255, 207, 110, 0.13);
        transform-style: preserve-3d;
        transform: translate3d(var(--card-x), var(--card-y), 0)
          rotateX(var(--rotate-x)) rotateY(var(--rotate-y));
        transition: transform 120ms ease-out;
        /* Hint the browser that this element will be transformed so
           it gets its own paint layer even when GPU compositing is
           off. In software mode this means we only re-rasterise the
           card region on pointer move, not the whole hero section. */
        will-change: transform;
        color: #f5f1e8;
        font-family: var(--font-body);
      }
      .hc-card::before,
      .hc-card::after {
        content: '';
        position: absolute;
        inset: 0;
        pointer-events: none;
      }
      .hc-card::before {
        /* The previous version used mix-blend-mode: screen to
           brighten the card face where the cursor hovers. Blend modes
           force a new compositing layer and re-composite on every
           paint, which is extremely expensive under software
           rendering. A straight rgba overlay looks ~identical on
           a dark card without the blend-mode cost. */
        background:
          linear-gradient(115deg, transparent 20%, rgba(255, 255, 255, 0.18) 45%, transparent 58%),
          radial-gradient(
            circle at var(--pointer-x) var(--pointer-y),
            rgba(255, 255, 255, 0.22),
            transparent 28%
          );
        /* Gated through load-progress so the sheen doesn't appear
           until the outline + shell have established the card. */
        opacity: calc(0.85 * var(--load-progress));
        transform: translateZ(60px);
      }
      .hc-card::after {
        background:
          repeating-linear-gradient(90deg, rgba(255, 255, 255, 0.04) 0 1px, transparent 1px 34px),
          repeating-linear-gradient(0deg, rgba(255, 255, 255, 0.03) 0 1px, transparent 1px 34px);
        mask-image: linear-gradient(180deg, transparent, black 18%, black 82%, transparent);
        -webkit-mask-image: linear-gradient(180deg, transparent, black 18%, black 82%, transparent);
        opacity: calc(0.35 * var(--load-progress));
        transform: translateZ(20px);
      }

      /* Holographic fill that wipes upward from the bottom of the
         card during the intro. clip-path does the wipe; the
         filter brightness/saturate pulse makes it feel like the
         card is cooling from a red-hot state back to resting. */
      .hc-card-shell {
        position: absolute;
        inset: 0;
        border-radius: inherit;
        pointer-events: none;
        background:
          linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.015)),
          linear-gradient(
            145deg,
            rgba(255, 207, 110, 0.18),
            rgba(124, 245, 208, 0.08) 44%,
            rgba(155, 209, 255, 0.18) 78%,
            rgba(255, 125, 182, 0.12)
          );
        opacity: 0;
        transform: translateZ(10px);
        clip-path: inset(100% 0 0 0 round 1.7rem);
        animation: hc-fillIn 1100ms cubic-bezier(0.2, 0.9, 0.2, 1) forwards 620ms;
      }

      /* SVG rect outline that draws around the card perimeter on
         load. The rect is 381 x 597 with rx=28, so its real
         perimeter is 2(325) + 2(541) + 2π(28) ≈ 1908 units.
         stroke-dasharray must be STRICTLY GREATER than the real
         perimeter or the last ~16% of the stroke ends up in the
         gap portion of the dash pattern and never gets drawn.
         An earlier comment here claimed 1600 was longer than the
         ~2050 perimeter — both figures were wrong, and the result
         was a stroke that drew ~84% of the way round and stopped.
         2200 comfortably exceeds the real perimeter for the
         current rect dimensions and leaves headroom if the
         aspect-ratio ever changes. Animation #1 draws the stroke,
         animation #2 dims it from full white to a subtle rim
         after the draw. */
      .hc-card-outline {
        position: absolute;
        inset: 0.08rem;
        width: calc(100% - 0.16rem);
        height: calc(100% - 0.16rem);
        overflow: visible;
        transform: translateZ(88px);
        pointer-events: none;
      }
      .hc-card-outline rect {
        fill: none;
        stroke: rgba(255, 245, 225, 0.95);
        stroke-width: 1.35;
        stroke-linecap: round;
        stroke-linejoin: round;
        stroke-dasharray: 2200;
        stroke-dashoffset: 2200;
        filter:
          drop-shadow(0 0 8px rgba(255, 255, 255, 0.2))
          drop-shadow(0 0 18px rgba(255, 207, 110, 0.18));
        animation:
          hc-drawOutline 900ms cubic-bezier(0.65, 0, 0.35, 1) forwards 80ms,
          hc-outlineFade 700ms ease forwards 1120ms;
      }

      /* Radial aura that pulses out from the card during the
         outline draw. Sits behind/around the card at negative
         inset so it spills past the card edges. */
      .hc-outline-glow {
        position: absolute;
        inset: -8%;
        border-radius: inherit;
        pointer-events: none;
        background: radial-gradient(
          circle at 50% 50%,
          rgba(255, 207, 110, 0.16),
          transparent 54%
        );
        filter: blur(18px);
        opacity: 0;
        transform: translateZ(92px) scale(0.94);
        animation: hc-glowPulse 1200ms ease forwards 260ms;
      }

      .hc-noise {
        position: absolute;
        inset: 0;
        /* mix-blend-mode: soft-light removed for the same perf reason
           as the sheen above. At 8% opacity on a dark card the visual
           result is close enough — a faint speckle — without paying
           the always-on blend cost. */
        opacity: calc(0.1 * var(--load-progress));
        background-image:
          radial-gradient(circle at 20% 20%, rgba(255, 255, 255, 0.6) 0 0.7px, transparent 0.8px),
          radial-gradient(circle at 70% 55%, rgba(255, 255, 255, 0.6) 0 0.7px, transparent 0.8px);
        background-size: 10px 10px, 13px 13px;
      }

      .hc-rings {
        position: absolute;
        inset: 18% -24% auto auto;
        width: 9rem;
        height: 9rem;
        transform: translateZ(42px);
        opacity: calc(0.7 * var(--load-progress));
        pointer-events: none;
      }
      .hc-rings::before,
      .hc-rings::after {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 1px solid rgba(255, 255, 255, 0.12);
      }
      .hc-rings::after {
        inset: 1.4rem;
        border-color: rgba(124, 245, 208, 0.2);
      }

      .hc-orb {
        position: absolute;
        border-radius: 50%;
        filter: blur(0.5px);
        pointer-events: none;
      }
      .hc-orb-one {
        width: 4.6rem;
        height: 4.6rem;
        right: -1rem;
        bottom: 4rem;
        background: radial-gradient(
          circle at 35% 35%,
          rgba(255, 255, 255, 0.8),
          rgba(255, 207, 110, 0.08) 38%,
          transparent 68%
        );
        transform: translateZ(84px);
        opacity: calc(0.95 * var(--load-progress));
      }
      .hc-orb-two {
        width: 2.9rem;
        height: 2.9rem;
        left: -1rem;
        top: 6rem;
        background: radial-gradient(
          circle at 40% 40%,
          rgba(255, 255, 255, 0.75),
          rgba(124, 245, 208, 0.12) 34%,
          transparent 70%
        );
        transform: translateZ(64px);
        opacity: calc(0.75 * var(--load-progress));
      }

      .hc-card-content {
        position: relative;
        z-index: 1;
        height: 100%;
        display: grid;
        grid-template-rows: auto 1fr auto;
        padding: 1.4rem;
        /* Content enters last — after the outline has drawn and the
           shell fill has wiped up — lifting from blur to sharp so
           the numbers, chip, and balance appear to be printed onto
           the card at the end of the reveal. */
        opacity: 0;
        animation: hc-contentLift 900ms cubic-bezier(0.2, 1, 0.22, 1) forwards 980ms;
      }

      .hc-topline,
      .hc-bottomline {
        display: flex;
        align-items: center;
        justify-content: space-between;
        letter-spacing: 0.16em;
        text-transform: uppercase;
        font-size: 0.64rem;
        color: rgba(255, 245, 225, 0.65);
        transform: translateZ(48px);
      }
      .hc-topline-right {
        white-space: nowrap;
      }

      /* Cards402 wordmark on the card face — SVG mask coloured with cream ink. */
      .hc-brand-wordmark {
        display: inline-block;
        width: 5.4rem;
        height: 1.25rem;
        background-color: #fff9ee;
        mask-image: url(/logo.svg);
        -webkit-mask-image: url(/logo.svg);
        mask-repeat: no-repeat;
        -webkit-mask-repeat: no-repeat;
        mask-size: contain;
        -webkit-mask-size: contain;
        mask-position: left center;
        -webkit-mask-position: left center;
        filter: drop-shadow(0 0 12px rgba(255, 207, 110, 0.25));
        flex-shrink: 0;
      }

      .hc-middle {
        display: grid;
        align-content: center;
        gap: 1rem;
      }

      .hc-chip {
        width: 3.1rem;
        height: 2.35rem;
        border-radius: 0.7rem;
        position: relative;
        background:
          linear-gradient(
            135deg,
            rgba(255, 245, 210, 0.95),
            rgba(255, 207, 110, 0.45)
          ),
          linear-gradient(90deg, rgba(0, 0, 0, 0.2), transparent);
        box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
        transform: translateZ(72px);
      }
      .hc-chip::before,
      .hc-chip::after {
        content: '';
        position: absolute;
        inset: 0.35rem;
        border: 1px solid rgba(90, 60, 0, 0.18);
        border-radius: 0.45rem;
      }
      .hc-chip::after {
        inset: auto 0.35rem 0.9rem;
        height: 1px;
        background: rgba(90, 60, 0, 0.18);
        border: 0;
      }

      .hc-balance {
        transform: translateZ(90px);
      }
      .hc-balance span {
        display: block;
        font-size: 0.62rem;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        color: rgba(255, 245, 225, 0.65);
        margin-bottom: 0.5rem;
      }
      .hc-balance strong {
        display: block;
        font-size: clamp(1.7rem, 3.4vw, 2.5rem);
        font-weight: 700;
        letter-spacing: -0.04em;
        color: #fff8ec;
        text-shadow: 0 0 2rem rgba(255, 207, 110, 0.24);
        font-family: var(--font-display);
        font-variation-settings: 'opsz' 144, 'SOFT' 20;
      }

      .hc-digits {
        display: grid;
        gap: 0.25rem;
        transform: translateZ(70px);
        font-family: var(--font-mono);
        color: rgba(255, 249, 235, 0.95);
      }
      .hc-digits .hc-label {
        font-size: 0.6rem;
        text-transform: uppercase;
        letter-spacing: 0.16em;
        color: rgba(255, 245, 225, 0.6);
      }
      .hc-digits .hc-value {
        font-size: 0.9rem;
        letter-spacing: 0.2em;
      }

      /* hc-floatCard (8s continuous filter loop cycling saturate +
         brightness) removed 2026-04-14 — it was causing a full
         card repaint every frame under software rendering, and the
         visual delta at 1.06× saturate was imperceptible anyway. */

      /* Card-wrap lifts from 2rem below with a blur + scale and settles. */
      @keyframes hc-wrapEnter {
        0% {
          opacity: 0;
          transform: translateY(2rem) scale(0.92);
          filter: blur(8px);
        }
        100% {
          opacity: 1;
          transform: translateY(0) scale(1);
          filter: blur(0);
        }
      }

      /* Stroke draws around the card perimeter from one corner. */
      @keyframes hc-drawOutline {
        0% {
          stroke-dashoffset: 2200;
          opacity: 1;
        }
        100% {
          stroke-dashoffset: 0;
          opacity: 1;
        }
      }

      /* After the draw completes, the outline dims to a subtle rim
         instead of sitting as a bright white line. */
      @keyframes hc-outlineFade {
        from { opacity: 1; }
        to { opacity: 0.18; }
      }

      /* Holographic fill wipes upward from the bottom, cooling from
         a bright saturate/brightness peak back to resting state. */
      @keyframes hc-fillIn {
        0% {
          opacity: 0.2;
          clip-path: inset(100% 0 0 0 round 1.7rem);
          filter: brightness(1.45) saturate(1.3);
        }
        100% {
          opacity: 1;
          clip-path: inset(0 0 0 0 round 1.7rem);
          filter: brightness(1) saturate(1);
        }
      }

      /* Radial aura scales from 78% to 108% while pulsing from 0 to
         ~0.95 then settling at 0.45. Feels like the card is
         radiating while the outline draws. */
      @keyframes hc-glowPulse {
        0% {
          opacity: 0;
          transform: translateZ(92px) scale(0.78);
        }
        46% {
          opacity: 0.95;
        }
        100% {
          opacity: 0.45;
          transform: translateZ(92px) scale(1.08);
        }
      }

      /* Content (chip + balance + digits) lifts in blurred → sharp
         once the card shell has arrived. */
      @keyframes hc-contentLift {
        0% {
          opacity: 0;
          transform: translateY(1rem) translateZ(20px) scale(0.98);
          filter: blur(10px);
        }
        100% {
          opacity: 1;
          transform: translateY(0) translateZ(0) scale(1);
          filter: blur(0);
        }
      }

      @media (max-width: 640px) {
        .hc-card {
          width: min(78vw, 19rem);
          border-radius: 1.4rem;
        }
        .hc-card-content {
          padding: 1.1rem;
        }
        .hc-rings {
          display: none;
        }
      }
      @media (max-width: 380px) {
        .hc-card {
          width: min(84vw, 16.5rem);
        }
      }
    `}</style>
  );
}
