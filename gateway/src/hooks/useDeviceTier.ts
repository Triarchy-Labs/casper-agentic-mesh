"use client";
import { useSyncExternalStore } from "react";

export type DeviceTier = "high" | "mid" | "low";

/**
 * Lusion-grade device capability detection.
 * Uses useSyncExternalStore to avoid setState-in-effect anti-pattern.
 * 
 * Classifies device into performance tiers for adaptive rendering:
 * - high: Desktop with dedicated GPU (full quality)
 * - mid: Tablets / powerful phones (reduced post-processing)  
 * - low: Older phones / weak GPUs (minimal effects)
 */

function getDeviceTier(): DeviceTier {
  if (typeof window === "undefined") return "high"; // SSR default

  const isTouchDevice = "ontouchstart" in window || navigator.maxTouchPoints > 0;
  const isSmallScreen = window.innerWidth < 768;
  const isMidScreen = window.innerWidth < 1200;
  const cores = navigator.hardwareConcurrency || 4;
  const dpr = window.devicePixelRatio || 1;
  const totalPixels = window.innerWidth * dpr * window.innerHeight * dpr;

  if (isTouchDevice && isSmallScreen) return "low";
  if (isTouchDevice && isMidScreen) return "mid";
  if (totalPixels > 8_000_000 && cores < 6) return "mid";
  return "high";
}

function getIsMobile(): boolean {
  if (typeof window === "undefined") return false;
  const isTouchDevice = "ontouchstart" in window || navigator.maxTouchPoints > 0;
  const isSmallScreen = window.innerWidth < 768;
  return isTouchDevice && isSmallScreen;
}

// Snapshot functions for useSyncExternalStore
const subscribeDummy = () => () => {};
const getServerTier = (): DeviceTier => "high";
const getServerMobile = (): boolean => false;

export function useDeviceTier(): DeviceTier {
  return useSyncExternalStore(subscribeDummy, getDeviceTier, getServerTier);
}

export function useIsMobile(): boolean {
  return useSyncExternalStore(subscribeDummy, getIsMobile, getServerMobile);
}
