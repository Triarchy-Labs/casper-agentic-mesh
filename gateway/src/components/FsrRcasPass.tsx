"use client";
/**
 * FsrRcasPass — Exact 1:1 port of Lusion Labs FSR RCAS (Robust Contrast-Adaptive Sharpening)
 * Source: lusion_formatted.js lines 42242-42291
 * Extracted: lusion_dump_chunks/24_fsr_smaa.md (lines 71-101)
 *
 * FSR RCAS is the second pass of AMD FidelityFX Super Resolution.
 * It applies adaptive sharpening that detects edges and only sharpens
 * where it won't create artifacts.
 *
 * For R3F, we use this WITHOUT EASU (the upscaler) since R3F Canvas dpr
 * already handles resolution scaling. RCAS alone gives the "crisp" look
 * that Lusion achieves — sharpening the slightly soft result of SMAA+Bloom.
 *
 * Lusion default: upscalerSharpness = 1
 */

import { forwardRef, useMemo } from "react";
import { Effect, BlendFunction } from "postprocessing";
import { Uniform } from "three";

// Exact GLSL from Lusion lines 42243-42249
const fragment = /* glsl */ `
  uniform float u_sharpness;

  // Lusion exact: FsrRcasCon (строка 42243)
  float FsrRcasCon(float sharpness) {
    return exp2(-sharpness);
  }

  void mainImage(const in vec4 inputColor, const in vec2 uv, out vec4 outputColor) {
    vec2 texelSize = 1.0 / resolution.xy;

    // 5-tap cross pattern: center + up/down/left/right (Lusion exact)
    vec3 e = inputColor.rgb;  // center
    vec3 b = texture2D(inputBuffer, uv + vec2(0.0, -texelSize.y)).rgb;  // up
    vec3 d = texture2D(inputBuffer, uv + vec2(-texelSize.x, 0.0)).rgb;  // left
    vec3 f = texture2D(inputBuffer, uv + vec2(texelSize.x, 0.0)).rgb;   // right
    vec3 h = texture2D(inputBuffer, uv + vec2(0.0, texelSize.y)).rgb;   // down

    // Luma for each sample (Rec. 709)
    float bL = dot(b, vec3(0.2126, 0.7152, 0.0722));
    float dL = dot(d, vec3(0.2126, 0.7152, 0.0722));
    float eL = dot(e, vec3(0.2126, 0.7152, 0.0722));
    float fL = dot(f, vec3(0.2126, 0.7152, 0.0722));
    float hL = dot(h, vec3(0.2126, 0.7152, 0.0722));

    // Min/max of cross neighborhood
    float nz = 0.25 * (bL + dL + fL + hL);
    float mn = min(min(bL, dL), min(eL, min(fL, hL)));
    float mx = max(max(bL, dL), max(eL, max(fL, hL)));

    // Adaptive lobe strength (Lusion exact)
    float range = mx - mn;
    float lobe = max(0.0, min(
      (mn + mn + mn + nz) / (4.0 * mx) - 1.0,
      0.0
    )) * FsrRcasCon(u_sharpness);
    lobe = max(-0.1875, lobe);  // Clamp to prevent over-sharpening

    // Weighted average (Lusion exact)
    vec3 col = (lobe * (b + d + h + f) + e) / (4.0 * lobe + 1.0);
    outputColor = vec4(col, 1.0);
  }
`;

class FsrRcasEffect extends Effect {
  constructor({ sharpness = 1.0 } = {}) {
    super("FsrRcasEffect", fragment, {
      blendFunction: BlendFunction.NORMAL,
      uniforms: new Map<string, Uniform<unknown>>([
        ["u_sharpness", new Uniform(sharpness)],
      ]),
    });
  }
}

interface FsrRcasPassProps {
  sharpness?: number; // Lusion default: 1.0
}

/**
 * R3F wrapper for FSR RCAS sharpening.
 * Place in <EffectComposer> AFTER SMAA (matching Lusion pipeline order).
 */
const FsrRcasPass = forwardRef(function FsrRcasPass(
  props: FsrRcasPassProps,
  ref
) {
  const { sharpness = 1.0 } = props;

  const effect = useMemo(() => {
    return new FsrRcasEffect({ sharpness });
  }, [sharpness]);

  return <primitive ref={ref} object={effect} dispose={null} />;
});

export default FsrRcasPass;
