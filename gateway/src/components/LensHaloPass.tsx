"use client";
/**
 * LensHaloPass — Exact 1:1 port of Lusion Labs Bloom Lens Halo
 * Source: lusion_formatted.js line 42436-42440
 *
 * Creates a chromatic ring halo around bright areas by:
 * 1. Inverting UVs through center (ghost UV)
 * 2. Projecting along direction with haloWidth
 * 3. Sampling RGB with per-channel chromatic shift
 * 4. Masking with aspect-correct smoothstep vignette
 *
 * Lusion defaults (Properties строки 28534-28538):
 * - haloWidth: 0.538
 * - haloRGBShift: 0.049
 * - haloStrength: 0.278
 * - haloMaskInner: 0.18
 * - haloMaskOuter: 0.0
 */

import { forwardRef, useMemo } from "react";
import { Effect, BlendFunction } from "postprocessing";
import { Uniform, Vector2 } from "three";
import { useThree, useFrame } from "@react-three/fiber";

// Exact GLSL from Lusion line 42436-42440
const fragment = /* glsl */ `
  uniform float u_haloWidth;
  uniform float u_haloRGBShift;
  uniform float u_haloStrength;
  uniform float u_haloMaskInner;
  uniform float u_haloMaskOuter;
  uniform vec2 u_aspect;

  void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
    // Lusion exact: line 42440
    vec2 toCenter = (uv - 0.5) * u_aspect;
    vec2 ghostUv = 1.0 - (toCenter + 0.5);
    vec2 ghostVec = (vec2(0.5) - ghostUv);
    vec2 direction = normalize(ghostVec);
    vec2 haloVec = direction * u_haloWidth;

    float weight = length(vec2(0.5) - fract(ghostUv + haloVec));
    weight = pow(1.0 - weight, 3.0);

    // Per-channel chromatic distortion (R shifts left, B shifts right)
    vec2 texelSize = 1.0 / resolution.xy;
    vec3 distortion = vec3(-texelSize.x, 0.0, texelSize.x) * u_haloRGBShift;
    vec2 haloUv = ghostUv + haloVec;

    // 3-channel chromatic ring sampling (Lusion exact)
    vec3 halo = vec3(
      texture2D(inputBuffer, haloUv + direction * distortion.r).r,
      texture2D(inputBuffer, haloUv + direction * distortion.g).g,
      texture2D(inputBuffer, haloUv + direction * distortion.b).b
    );

    // Masked by aspect-correct vignette (Lusion exact)
    float mask = smoothstep(u_haloMaskInner, u_haloMaskOuter, length(toCenter));

    outputColor = vec4(inputColor.rgb + halo * u_haloStrength * mask, inputColor.a);
  }
`;

class LensHaloEffect extends Effect {
  constructor({
    haloWidth = 0.538,
    haloRGBShift = 0.049,
    haloStrength = 0.278,
    haloMaskInner = 0.18,
    haloMaskOuter = 0.0,
  } = {}) {
    super("LensHaloEffect", fragment, {
      blendFunction: BlendFunction.NORMAL,
      uniforms: new Map<string, Uniform<unknown>>([
        ["u_haloWidth", new Uniform(haloWidth)],
        ["u_haloRGBShift", new Uniform(haloRGBShift)],
        ["u_haloStrength", new Uniform(haloStrength)],
        ["u_haloMaskInner", new Uniform(haloMaskInner)],
        ["u_haloMaskOuter", new Uniform(haloMaskOuter)],
        ["u_aspect", new Uniform(new Vector2(1, 1))],
      ]),
    });
  }
}

interface LensHaloPassProps {
  haloWidth?: number;
  haloRGBShift?: number;
  haloStrength?: number;
}

/**
 * R3F wrapper for Lusion Lens Halo.
 * Place inside <EffectComposer> AFTER Bloom — adds chromatic ring around bright areas.
 */
const LensHaloPass = forwardRef(function LensHaloPass(
  props: LensHaloPassProps,
  ref
) {
  const {
    haloWidth = 0.538,
    haloRGBShift = 0.049,
    haloStrength = 0.278,
  } = props;

  const { size } = useThree();

  const effect = useMemo(() => {
    return new LensHaloEffect({ haloWidth, haloRGBShift, haloStrength });
  }, [haloWidth, haloRGBShift, haloStrength]);

  // Update aspect ratio every frame
  useFrame(() => {
    const aspect = effect.uniforms.get("u_aspect")!.value as Vector2;
    aspect.set(size.width / size.height, 1.0);
  });

  return <primitive ref={ref} object={effect} dispose={null} />;
});

export default LensHaloPass;
