"use client";
/**
 * ScreenPaintDistortion — Exact 1:1 port of Lusion Labs ScreenPaint distortion overlay
 * Source: lusion_formatted.js lines 42644-42691
 * Extracted: lusion_dump_chunks/23_ease_bluenoise_transition.md (lines 80-114)
 *
 * Now uses the REAL LDR_RGB1_0.png blue noise texture (128×128, CC0 Christoph Peters)
 * downloaded directly from labs.lusion.co/assets/textures/LDR_RGB1_0.png
 *
 * Implements:
 * 1. 4-tap motion blur along velocity direction
 * 2. Blue noise jitter using REAL texture for temporal anti-aliasing
 * 3. Chromatic RGB shift proportional to velocity magnitude
 *
 * Lusion JS Defaults (lines 42648-42652):
 * - amount: 20, rgbShift: 0.5, multiplier: 5, colorMultiplier: 10, shade: 1.25
 */

import { forwardRef, useMemo, useEffect } from "react";
import { Effect, BlendFunction } from "postprocessing";
import {
  Uniform,
  Vector2,
  TextureLoader,
  NearestFilter,
  RepeatWrapping,
  type Texture,
} from "three";
import { useFrame } from "@react-three/fiber";

// Exact GLSL from Lusion lines 42640-42643
// Uses REAL blue noise texture instead of hash approximation
const fragment = /* glsl */ `
  uniform sampler2D u_screenPaintTexture;
  uniform vec2 u_screenPaintTexelSize;
  uniform float u_amount;
  uniform float u_rgbShift;
  uniform float u_multiplier;
  uniform float u_colorMultiplier;
  uniform float u_shade;

  // Real blue noise texture (Lusion: LDR_RGB1_0.png, 128×128)
  uniform sampler2D u_blueNoiseTexture;
  uniform vec2 u_blueNoiseTexelSize;
  uniform vec2 u_blueNoiseCoordOffset;

  // Lusion exact getBlueNoise (строка 3206 in index.f4419199.js)
  vec3 getBlueNoise(vec2 coord) {
    return texture2D(u_blueNoiseTexture, coord * u_blueNoiseTexelSize + u_blueNoiseCoordOffset).rgb;
  }

  void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
    vec3 bnoise = getBlueNoise(gl_FragCoord.xy + vec2(17., 29.));
    vec4 data = texture2D(u_screenPaintTexture, uv);
    float weight = (data.z + data.w) * 0.5;
    vec2 vel = (0.5 - data.xy - 0.001) * 2.0 * weight;

    // 4-tap motion blur along velocity from paint (Lusion exact)
    vec4 color = vec4(0.0);
    vec2 velocity = vel * u_amount / 4.0 * u_screenPaintTexelSize * u_multiplier;
    vec2 sampleUv = uv + bnoise.xy * velocity;  // blue noise jitter (temporal AA)
    for (int i = 0; i < 4; i++) {
      color += texture2D(inputBuffer, sampleUv);
      sampleUv += velocity;
    }
    color /= 4.0;

    // Chromatic RGB shift proportional to velocity (Lusion exact)
    color.rgb += sin(vec3(vel.x + vel.y) * 40.0 + vec3(0.0, 2.0, 4.0) * u_rgbShift)
      * smoothstep(0.4, -0.9, weight) * u_shade
      * max(abs(vel.x), abs(vel.y)) * u_colorMultiplier;

    outputColor = color;
  }
`;

const BLUE_NOISE_SIZE = 128; // Lusion: TEXTURE_SIZE = 128

class ScreenPaintDistortionEffect extends Effect {
  constructor({
    screenPaintTexture = null as Texture | null,
    amount = 20,
    rgbShift = 0.5,
    multiplier = 5,
    colorMultiplier = 10,
    shade = 1.25,
  } = {}) {
    super("ScreenPaintDistortionEffect", fragment, {
      blendFunction: BlendFunction.NORMAL,
      uniforms: new Map<string, Uniform<unknown>>([
        ["u_screenPaintTexture", new Uniform(screenPaintTexture)],
        ["u_screenPaintTexelSize", new Uniform(new Vector2(1 / 256, 1 / 256))],
        ["u_amount", new Uniform(amount)],
        ["u_rgbShift", new Uniform(rgbShift)],
        ["u_multiplier", new Uniform(multiplier)],
        ["u_colorMultiplier", new Uniform(colorMultiplier)],
        ["u_shade", new Uniform(shade)],
        // Blue noise uniforms (Lusion: BlueNoise class, строка 42606)
        ["u_blueNoiseTexture", new Uniform(null)],
        ["u_blueNoiseTexelSize", new Uniform(new Vector2(1 / BLUE_NOISE_SIZE, 1 / BLUE_NOISE_SIZE))],
        ["u_blueNoiseCoordOffset", new Uniform(new Vector2(0, 0))],
      ]),
    });
  }
}

interface ScreenPaintDistortionProps {
  paintTexture: Texture | null;
  amount?: number;
  rgbShift?: number;
  multiplier?: number;
  colorMultiplier?: number;
  shade?: number;
}

/**
 * R3F wrapper for Lusion ScreenPaint Distortion.
 * Loads the REAL LDR_RGB1_0.png blue noise texture on mount.
 */
const ScreenPaintDistortion = forwardRef(function ScreenPaintDistortion(
  props: ScreenPaintDistortionProps,
  ref
) {
  const {
    paintTexture,
    amount = 20,
    rgbShift = 0.5,
    multiplier = 5,
    colorMultiplier = 10,
    shade = 1.25,
  } = props;

  const effect = useMemo(() => {
    return new ScreenPaintDistortionEffect({
      screenPaintTexture: paintTexture,
      amount,
      rgbShift,
      multiplier,
      colorMultiplier,
      shade,
    });
  }, [paintTexture, amount, rgbShift, multiplier, colorMultiplier, shade]);

  // Load REAL blue noise texture on mount (Lusion: BlueNoise.preInit())
  useEffect(() => {
    const loader = new TextureLoader();
    loader.load("/LDR_RGB1_0.png", (tex) => {
      // Lusion exact: NearestFilter + RepeatWrapping (строка 42627-42630)
      tex.generateMipmaps = false;
      tex.minFilter = NearestFilter;
      tex.magFilter = NearestFilter;
      tex.wrapS = RepeatWrapping;
      tex.wrapT = RepeatWrapping;
      effect.uniforms.get("u_blueNoiseTexture")!.value = tex;
    });
  }, [effect]);

  // Update every frame (Lusion: BlueNoise.update())
  useFrame(() => {
    if (paintTexture) {
      effect.uniforms.get("u_screenPaintTexture")!.value = paintTexture;
    }
    // Lusion exact: каждый кадр — случайный offset (NOT increment!)
    const offset = effect.uniforms.get("u_blueNoiseCoordOffset")!.value as Vector2;
    offset.set(Math.random(), Math.random());
  });

  return <primitive ref={ref} object={effect} dispose={null} />;
});

export default ScreenPaintDistortion;
