"use client";
import React from "react";
import { AnimatePresence, motion } from "framer-motion";

interface CardCTAProps {
  label: string;
  show: boolean;
}

const EASE = [0.32, 0.72, 0, 1] as [number, number, number, number];

const arrowVariants = {
  hidden: { width: 0, scale: 0, opacity: 0 },
  visible: {
    width: 24,
    scale: 1,
    opacity: 1,
    transition: { duration: 0.25, ease: EASE },
  },
  exit: {
    width: 0,
    scale: 0,
    opacity: 0,
    transition: { duration: 0.2, ease: EASE, delay: 0.08 },
  },
};

const letterVariants = {
  hidden: { x: "-110%", opacity: 0 },
  visible: (i: number) => ({
    x: "0%",
    opacity: 1,
    transition: {
      duration: 0.22,
      ease: EASE,
      delay: 0.06 + i * 0.03,
    },
  }),
  exit: (i: number) => ({
    x: "-110%",
    opacity: 0,
    transition: {
      duration: 0.15,
      ease: EASE,
      delay: i * 0.015,
    },
  }),
};

export function CardCTA({ label, show }: CardCTAProps) {
  const letters = label.split("");

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          className="card-cta"
          initial="hidden"
          animate="visible"
          exit="exit"
          style={{
            position: "absolute",
            top: 0,
            right: 0,
            display: "flex",
            alignItems: "center",
            gap: 0,
            zIndex: 20,
            pointerEvents: "none",
          }}
        >
          {/* Staggered letter reveal */}
          <motion.span
            style={{
              display: "flex",
              overflow: "hidden",
              paddingRight: 6,
            }}
          >
            {letters.map((char, i) => (
              <motion.span
                key={`${char}-${i}`}
                custom={i}
                variants={letterVariants}
                style={{
                  display: "inline-block",
                  fontFamily: "var(--font-mono, monospace)",
                  fontSize: 11,
                  fontWeight: 600,
                  letterSpacing: "0.12em",
                  color: "rgba(255,255,255,0.85)",
                  lineHeight: "24px",
                  whiteSpace: "pre",
                }}
              >
                {char}
              </motion.span>
            ))}
          </motion.span>

          {/* Arrow square */}
          <motion.div
            variants={arrowVariants}
            style={{
              height: 24,
              background: "#fff",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              overflow: "hidden",
              flexShrink: 0,
              borderRadius: 0,
            }}
          >
            <svg
              viewBox="0 0 8 8"
              fill="none"
              width={12}
              height={12}
              style={{ transform: "rotate(-45deg)" }}
            >
              <path
                d="M2.49 7.1L5.76 3.8L2.49.51L4.13.5L7.43 3.8L4.13 7.1Z"
                fill="black"
              />
              <path d="M0 3.25h5.78v1.1H0z" fill="black" />
            </svg>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
