"use client";
/**
 * BrownianMotionCamera — Exact 1:1 port of Lusion Labs camera shake system
 * Source: lusion_formatted.js lines 48928-49034
 * Extracted: lusion_dump_chunks/22_camera_shake.md
 *
 * 6-channel FBM (Fractional Brownian Motion) camera shake:
 * 3 channels for position (x, y, z) + 3 channels for rotation (x, y, z=0)
 *
 * Lusion Defaults (from Properties):
 * - cameraShakePositionStrength: 0.2
 * - cameraShakePositionSpeed: 0.12
 * - cameraShakeRotationStrength: 0.0016
 * - cameraShakeRotationSpeed: 0.3
 */

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

// Lusion exact: Simple1DNoise (строки 48928-48958)
class Simple1DNoise {
  static MAX_VERTICES = 256;
  _scale = 1;
  _amplitude = 1;
  _r: number[] = [];

  constructor() {
    for (let i = 0; i < Simple1DNoise.MAX_VERTICES; i++) {
      this._r[i] = Math.random() - 0.5;
    }
  }

  getVal(x: number): number {
    const t = x * this._scale;
    const i = Math.floor(t);
    const f = t - i;
    // Hermite interpolation (exact from Lusion)
    const s = f * f * (3 - 2 * f);
    const a = this._r[i & 255];
    const b = this._r[(i + 1) & 255];
    return (a + s * (b - a)) * this._amplitude;
  }
}

// Lusion exact FBM_NORM = 1 / 0.75 = 1.333...
const FBM_NORM = 1 / 0.75;

function fbm(noise: Simple1DNoise, x: number, octaves: number): number {
  let sum = 0;
  let amp = 0.5;
  for (let i = 0; i < octaves; i++) {
    sum += amp * noise.getVal(x);
    x *= 2;
    amp *= 0.5;
  }
  return sum;
}

interface BrownianMotionCameraProps {
  // Lusion defaults from Properties
  positionStrength?: number;   // 0.2
  positionSpeed?: number;      // 0.12
  rotationStrength?: number;   // 0.0016
  rotationSpeed?: number;      // 0.3
  octaves?: number;            // 3
  enabled?: boolean;
}

/**
 * Drop into <Canvas> to add Lusion-style subtle camera shake.
 * Does NOT change camera position in world — only applies micro-offsets.
 */
export default function BrownianMotionCamera({
  positionStrength = 0.2,
  positionSpeed = 0.12,
  rotationStrength = 0.0016,
  rotationSpeed = 0.3,
  octaves = 3,
  enabled = true,
}: BrownianMotionCameraProps) {
  // 6 independent noise channels (Lusion exact)
  const noises = useMemo(() => Array.from({ length: 6 }, () => new Simple1DNoise()), []);
  const times = useRef([0, 0, 0, 0, 0, 0]);
  const offsetPos = useMemo(() => new THREE.Vector3(), []);
  const offsetRot = useMemo(() => new THREE.Euler(), []);

  useFrame((state, delta) => {
    if (!enabled) return;
    const cam = state.camera;
    const t = times.current;

    // Position channels (Lusion: _positionFrequency * dt)
    t[0] += positionSpeed * delta;
    t[1] += positionSpeed * delta;
    t[2] += positionSpeed * delta;

    offsetPos.set(
      fbm(noises[0], t[0], octaves) * positionStrength * FBM_NORM,
      fbm(noises[1], t[1], octaves) * positionStrength * FBM_NORM,
      fbm(noises[2], t[2], octaves) * positionStrength * FBM_NORM,
    );

    // Rotation channels (Lusion: _rotationFrequency * dt, Z rotation=0)
    t[3] += rotationSpeed * delta;
    t[4] += rotationSpeed * delta;
    t[5] += rotationSpeed * delta;

    offsetRot.set(
      fbm(noises[3], t[3], octaves) * rotationStrength * FBM_NORM,
      fbm(noises[4], t[4], octaves) * rotationStrength * FBM_NORM,
      0, // Lusion: _rotationScale.z = 0 (no roll)
    );

    // Apply micro-offsets to camera
    cam.position.x += offsetPos.x;
    cam.position.y += offsetPos.y;
    cam.position.z += offsetPos.z;

    cam.rotation.x += offsetRot.x;
    cam.rotation.y += offsetRot.y;
  });

  return null;
}
