"use client";
/**
 * LusionFinalPass — Exact 1:1 port of Lusion Labs Final composite shader
 * Source: lusion_formatted.js lines 42693-42790
 * Extracted: lusion_dump_chunks/06_final_pass_shader.glsl
 *
 * This is the LAST pass before screen output. Implements:
 * - Saturation / Contrast / Brightness adjustment
 * - Color Dodge + Screen tint blend modes
 * - Aspect-correct vignette
 * - hash13 temporal dithering (1-bit noise / 255.0 to break banding)
 *
 * Every constant, function name, and formula is verified against the dump.
 */

import { forwardRef, useMemo } from "react";
import { Effect, BlendFunction } from "postprocessing";
import { Uniform, Vector2 } from "three";
import { useFrame, useThree } from "@react-three/fiber";

// Exact GLSL from Lusion lines 42693-42790
const fragment = /* glsl */ `
  uniform vec3 u_bgColor;
  uniform float u_opacity;
  uniform float u_vignetteFrom;
  uniform float u_vignetteTo;
  uniform vec2 u_vignetteAspect;
  uniform vec3 u_vignetteColor;
  uniform float u_saturation;
  uniform float u_contrast;
  uniform float u_brightness;
  uniform float u_invert;
  uniform vec3 u_tintColor;
  uniform float u_tintOpacity;
  uniform float u_ditherSeed;

  // Lusion exact hash13 (строка 42693) — .yzx + 33.33
  float hash13(vec3 p3) {
    p3 = fract(p3 * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
  }

  // Photoshop "Screen" blend (строка 42693)
  vec3 screen(vec3 cb, vec3 cs) {
    return cb + cs - (cb * cs);
  }

  // Photoshop "Color Dodge" blend (строка 42693)
  vec3 colorDodge(vec3 cb, vec3 cs) {
    return mix(
      min(vec3(1.0), cb / (1.0 - cs)),
      vec3(1.0),
      step(vec3(1.0), cs)
    );
  }

  void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
    vec3 color = inputColor.rgb;

    // Luminance (Rec. 601)
    float luma = dot(color, vec3(0.299, 0.587, 0.114));

    // Saturation adjustment (u_saturation=0 = neutral, from Lusion line 42703: saturation-1)
    color = mix(vec3(luma), color, 1.0 + u_saturation);

    // Contrast adjustment centered at 0.5 (u_contrast=0 = neutral)
    color = 0.5 + (1.0 + u_contrast) * (color - 0.5);

    // Brightness (u_brightness=0 = neutral, from Lusion line 42705: brightness-1)
    color += u_brightness;

    // Invert (u_invert=0 = off)
    color = mix(color, 1.0 - color, u_invert);

    // Tint: Color Dodge + Screen blend (Lusion line 42707: tintOpacity=1 default, but properties override)
    color = mix(color, screen(colorDodge(color, u_tintColor), u_tintColor), u_tintOpacity);

    // VIGNETTE — aspect-correct (Lusion lines 42699-42700, 42790)
    float d = length((uv - 0.5) * u_vignetteAspect) * 2.0;
    color = mix(color, u_vignetteColor, smoothstep(u_vignetteFrom, u_vignetteTo, d));

    // Final composite: bg mix + 1-bit dither noise to break banding (Lusion line 42790)
    outputColor = vec4(
      mix(u_bgColor, color, u_opacity) + hash13(vec3(gl_FragCoord.xy, u_ditherSeed)) / 255.0,
      1.0
    );
  }
`;

class LusionFinalEffect extends Effect {
  constructor({
    bgColor = [0.004, 0.008, 0.004], // #010201
    opacity = 1.0,
    vignetteFrom = 0.6,    // Lusion EXACT (строка 42699)
    vignetteTo = 1.6,      // Lusion EXACT (строка 42700)
    vignetteColor = [0, 0, 0],
    saturation = 0,         // Lusion: saturation - 1 = 0 (neutral)
    contrast = 0,           // Lusion: contrast = 0 (neutral)
    brightness = 0,         // Lusion: brightness - 1 = 0 (neutral)
    invert = 0,
    tintColor = [1, 1, 1],
    tintOpacity = 0.05,
  } = {}) {
    super("LusionFinalEffect", fragment, {
      blendFunction: BlendFunction.NORMAL,
      uniforms: new Map<string, Uniform<unknown>>([
        ["u_bgColor", new Uniform(bgColor)],
        ["u_opacity", new Uniform(opacity)],
        ["u_vignetteFrom", new Uniform(vignetteFrom)],
        ["u_vignetteTo", new Uniform(vignetteTo)],
        ["u_vignetteAspect", new Uniform(new Vector2(1, 1))],
        ["u_vignetteColor", new Uniform(vignetteColor)],
        ["u_saturation", new Uniform(saturation)],
        ["u_contrast", new Uniform(contrast)],
        ["u_brightness", new Uniform(brightness)],
        ["u_invert", new Uniform(invert)],
        ["u_tintColor", new Uniform(tintColor)],
        ["u_tintOpacity", new Uniform(tintOpacity)],
        ["u_ditherSeed", new Uniform(0)],
      ]),
    });
  }
}

/**
 * R3F wrapper for Lusion Final Pass.
 * Drop into <EffectComposer> as the LAST effect before screen.
 */
const LusionFinalPass = forwardRef(function LusionFinalPass(
  props: {
    vignetteFrom?: number;
    vignetteTo?: number;
    saturation?: number;
    contrast?: number;
    brightness?: number;
    tintOpacity?: number;
    theme?: "dark" | "light";
  },
  ref
) {
  const {
    vignetteFrom = 0.6,
    vignetteTo = 1.6,
    saturation = 0,
    contrast = 0,
    brightness = 0,
    tintOpacity = 0.05,
    theme = "dark",
  } = props;

  const { size } = useThree();

  const effect = useMemo(() => {
    return new LusionFinalEffect({
      bgColor: theme === "dark" ? [0.004, 0.008, 0.004] : [0.98, 0.98, 0.98],
      vignetteFrom,
      vignetteTo,
      vignetteColor: theme === "dark" ? [0, 0, 0] : [0.95, 0.95, 0.95],
      saturation,
      contrast,
      brightness,
      tintOpacity,
    });
  }, [theme, vignetteFrom, vignetteTo, saturation, contrast, brightness, tintOpacity]);

  // Update dither seed every frame (Lusion line 42790: Math.random() * 1000)
  useFrame(() => {
    effect.uniforms.get("u_ditherSeed")!.value = Math.random() * 1000;

    // Lusion vignette aspect formula (Postprocessing.js line 71)
    const w = size.width;
    const h = size.height;
    const aspectY = h / Math.sqrt(w * w + h * h) * 2;
    effect.uniforms.get("u_vignetteAspect")!.value.set((w / h) * aspectY, aspectY);

    // Update theme-dependent uniforms
    effect.uniforms.get("u_bgColor")!.value = theme === "dark"
      ? [0.004, 0.008, 0.004]
      : [0.98, 0.98, 0.98];
    effect.uniforms.get("u_vignetteColor")!.value = theme === "dark"
      ? [0, 0, 0]
      : [0.95, 0.95, 0.95];
  });

  return <primitive ref={ref} object={effect} dispose={null} />;
});

export default LusionFinalPass;
