"use client";

import React, { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface GlitchWormProps {
    progress: number; // 0 to 100
}

/**
 * GlitchWormProgress — Ingestion progress bar
 * 
 * OPTIMIZATION: Replaced 28 × Framer Motion `repeat: Infinity` animations
 * with a single CSS @keyframes animation. Each Framer Motion instance spawns
 * its own requestAnimationFrame loop — 28 parallel RAF = 16fps on iPhone.
 * CSS animations are GPU-composited natively, zero JS overhead.
 */
export function GlitchWormProgress({ progress }: GlitchWormProps) {
    const TOTAL_CUBES = 28;
    const safeProgress = Math.max(0, Math.min(100, progress));
    const filledCount = Math.floor((safeProgress / 100) * TOTAL_CUBES);
    
    const [glitching, setGlitching] = useState(false);

    useEffect(() => {
        if (safeProgress === 100) {
            setTimeout(() => setGlitching(true), 0);
        } else {
            setTimeout(() => setGlitching(false), 0);
        }
    }, [safeProgress]);

    return (
        <div className="relative w-full max-w-[400px] h-20 flex flex-col items-center justify-center font-mono">
            
            <AnimatePresence mode="wait">
                {glitching ? (
                    <motion.div 
                        key="override-text"
                        initial={{ opacity: 0, scale: 0.8, filter: "blur(10px)" }}
                        animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
                        exit={{ opacity: 0 }}
                        className="absolute inset-0 flex items-center justify-center z-50 text-[#00ff41] font-bold text-center tracking-widest text-[14px]"
                        style={{ textShadow: "0 0 20px #00ff41, 0 0 10px #00ff41" }}
                    >
                        <motion.span
                            animate={{ x: [-2, 2, -2, 0], y: [1, -1, 1, 0] }}
                            transition={{ duration: 0.2, repeat: 3 }}
                        >
                            [ OVERRIDE COMPLETE. EXCELLENT. ]
                        </motion.span>
                    </motion.div>
                ) : (
                    <motion.div 
                        key="worm-bar"
                        exit={{ opacity: 0, scale: 1.1, filter: "brightness(200%)" }}
                        className="flex flex-col items-center gap-2 w-full"
                    >
                        <div className="text-[10px] text-white/30 uppercase tracking-[0.2em] flex justify-between w-full pb-1 border-b border-white/5">
                            <span>Ingestion Worm</span>
                            <span className="text-[#00ff41] drop-shadow-[0_0_5px_#00ff41]">{Math.round(safeProgress)}%</span>
                        </div>
                        <div className="flex gap-1 justify-between w-full relative">
                            {Array.from({ length: TOTAL_CUBES }).map((_, i) => {
                                const isFilled = i < filledCount;
                                const isHead = i === filledCount;
                                
                                return (
                                    <div 
                                        key={i} 
                                        className={`w-full max-w-[12px] h-4 rounded-[1px] relative overflow-hidden ${
                                            isFilled ? "bg-[#00ff41] shadow-[0_0_10px_rgba(0,255,65,0.6)]" : "bg-white/5 border border-white/10"
                                        }`}
                                    >
                                        {/* The Glitch Head Eating Effect — CSS animation, not Framer */}
                                        {isHead && (
                                            <div 
                                                className="absolute inset-0 bg-white"
                                                style={{
                                                    animation: "wormHeadBlink 1s ease-in-out infinite",
                                                }}
                                            />
                                        )}
                                        {/* Tail Pulse — CSS animation with staggered delay (was Framer repeat: Infinity) */}
                                        {isFilled && !isHead && (
                                            <div 
                                                className="absolute inset-0 bg-white/20"
                                                style={{
                                                    animation: "wormShineSweep 1.5s linear infinite",
                                                    animationDelay: `${i * 0.05}s`,
                                                }}
                                            />
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

        </div>
    );
}
