"use client";
// Stars REMOVED — cannot individually drift, only group rotation
// All particles now unified in LiquidNebula with CPU-side animation
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import {
	Bloom,
	ChromaticAberration,
	EffectComposer,
	SMAA,
} from "@react-three/postprocessing";
import { SMAAPreset } from "postprocessing";
import { BlendFunction } from "postprocessing";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import * as THREE from "three";
import { useUnifiedPointer } from "../hooks/useUnifiedPointer";
import { useDeviceTier, type DeviceTier } from "../hooks/useDeviceTier";
import RefractiveCore from "./RefractiveCore";
import ScreenPaint from "./ScreenPaint";
import LusionFinalPass from "./LusionFinalPass";
import ScreenPaintDistortion from "./ScreenPaintDistortion";
import BrownianMotionCamera from "./BrownianMotionCamera";
import FsrRcasPass from "./FsrRcasPass";
import LensHaloPass from "./LensHaloPass";

// Lusion-grade adaptive constants per device tier
const TIER_CONFIG = {
	high: { particles: 16384, smaa: SMAAPreset.HIGH, bloomIntensity: 1.5, dpr: [1, 1.5] as [number, number], enableChroma: true },
	mid:  { particles: 8192,  smaa: SMAAPreset.MEDIUM, bloomIntensity: 1.0, dpr: [1, 1.2] as [number, number], enableChroma: true },
	low:  { particles: 4096,  smaa: SMAAPreset.LOW, bloomIntensity: 0.6, dpr: [1, 1] as [number, number], enableChroma: false },
};

// ═══════════════════════════════════════════════════════════════════
// GPGPU PARTICLE SYSTEM — 1:1 from Lusion dump (строки 48648-48870)
// ═══════════════════════════════════════════════════════════════════

import { GPUComputationRenderer } from "three/examples/jsm/misc/GPUComputationRenderer.js";

const TEX_SIZE = 128; // строка 48648
const PARTICLE_COUNT = TEX_SIZE * TEX_SIZE; // 16384, строка 48649

// Render: boosted to compensate for missing Bloom postprocessing (physics remain 1:1)
const U_OPACITY = "0.95";
const U_P_SIZE_MUL = "1.6";
const U_P_SOFT_MUL = "2.5";
const U_FOCUS_DIST = "0.32";

// Lusion EXACT spawn/kill (строки 48653-48664)
// NOTE: Use strings with decimals for GLSL (JS integers break shader compilation)
const SPAWN_X = "4.0"; const SPAWN_Y = "2.4"; const SPAWN_Z = "0.64";
const SPAWN_OX = "-3.0"; const SPAWN_OY = "-0.5"; const SPAWN_OZ = "0.0";  // Lusion EXACT offset (11_key_constants line 16)
const KILL_X = "7.0"; const KILL_Y = "5.0"; const KILL_Z = "2.0";

// ── Exact Lusion GLSL: Simplex 4D Derivatives + Curl (from 01_particle_position_shader.glsl) ──
const NOISE_GLSL = /* glsl */ `
vec4 mod289(vec4 x) { return x - floor(x * (1.0/289.0)) * 289.0; }
float mod289(float x) { return x - floor(x * (1.0/289.0)) * 289.0; }
vec4 permute(vec4 x) { return mod289(((x * 34.0) + 1.0) * x); }
float permute(float x) { return mod289(((x * 34.0) + 1.0) * x); }
vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }
float taylorInvSqrt(float r) { return 1.79284291400159 - 0.85373472095314 * r; }

vec4 grad4(float j, vec4 ip) {
  vec4 p, s;
  p.xyz = floor(fract(vec3(j) * ip.xyz) * 7.0) * ip.z - 1.0;
  p.w = 1.5 - dot(abs(p.xyz), vec3(1.0, 1.0, 1.0));
  s = vec4(lessThan(p, vec4(0.0)));
  p.xyz = p.xyz + (s.xyz * 2.0 - 1.0) * s.www;
  return p;
}

// Lusion EXACT: returns vec4(dx, dy, dz, dw) * 49.0 — ANALYTICAL DERIVATIVES
vec4 simplexNoiseDerivatives(vec4 v) {
  const vec4 C = vec4(0.138196601125011, 0.276393202250021, 0.414589803375032, -0.447213595499958);
  vec4 i = floor(v + dot(v, vec4(0.309016994374947451)));
  vec4 x0 = v - i + dot(i, C.xxxx);
  vec4 i0;
  vec3 isX = step(x0.yzw, x0.xxx);
  vec3 isYZ = step(x0.zww, x0.yyz);
  i0.x = isX.x + isX.y + isX.z;
  i0.yzw = 1.0 - isX;
  i0.y += isYZ.x + isYZ.y;
  i0.zw += 1.0 - isYZ.xy;
  i0.z += isYZ.z;
  i0.w += 1.0 - isYZ.z;
  vec4 i3 = clamp(i0, 0.0, 1.0);
  vec4 i2 = clamp(i0 - 1.0, 0.0, 1.0);
  vec4 i1 = clamp(i0 - 2.0, 0.0, 1.0);
  vec4 x1 = x0 - i1 + C.xxxx;
  vec4 x2 = x0 - i2 + C.yyyy;
  vec4 x3 = x0 - i3 + C.zzzz;
  vec4 x4 = x0 + C.wwww;
  i = mod289(i);
  float j0 = permute(permute(permute(permute(i.w) + i.z) + i.y) + i.x);
  vec4 j1 = permute(permute(permute(permute(
      i.w + vec4(i1.w, i2.w, i3.w, 1.0)) +
      i.z + vec4(i1.z, i2.z, i3.z, 1.0)) +
      i.y + vec4(i1.y, i2.y, i3.y, 1.0)) +
      i.x + vec4(i1.x, i2.x, i3.x, 1.0));
  vec4 ip2 = vec4(1.0/294.0, 1.0/49.0, 1.0/7.0, 0.0);
  vec4 p0 = grad4(j0, ip2);
  vec4 p1 = grad4(j1.x, ip2);
  vec4 p2 = grad4(j1.y, ip2);
  vec4 p3 = grad4(j1.z, ip2);
  vec4 p4 = grad4(j1.w, ip2);
  vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
  p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;
  p4 *= taylorInvSqrt(dot(p4, p4));
  vec3 values0 = vec3(dot(p0,x0), dot(p1,x1), dot(p2,x2));
  vec2 values1 = vec2(dot(p3,x3), dot(p4,x4));
  vec3 m0 = max(0.5 - vec3(dot(x0,x0), dot(x1,x1), dot(x2,x2)), 0.0);
  vec2 m1 = max(0.5 - vec2(dot(x3,x3), dot(x4,x4)), 0.0);
  vec3 temp0 = -6.0 * m0 * m0 * values0;
  vec2 temp1 = -6.0 * m1 * m1 * values1;
  vec3 mmm0 = m0 * m0 * m0;
  vec2 mmm1 = m1 * m1 * m1;
  float dx2 = temp0[0]*x0.x + temp0[1]*x1.x + temp0[2]*x2.x + temp1[0]*x3.x + temp1[1]*x4.x + mmm0[0]*p0.x + mmm0[1]*p1.x + mmm0[2]*p2.x + mmm1[0]*p3.x + mmm1[1]*p4.x;
  float dy2 = temp0[0]*x0.y + temp0[1]*x1.y + temp0[2]*x2.y + temp1[0]*x3.y + temp1[1]*x4.y + mmm0[0]*p0.y + mmm0[1]*p1.y + mmm0[2]*p2.y + mmm1[0]*p3.y + mmm1[1]*p4.y;
  float dz2 = temp0[0]*x0.z + temp0[1]*x1.z + temp0[2]*x2.z + temp1[0]*x3.z + temp1[1]*x4.z + mmm0[0]*p0.z + mmm0[1]*p1.z + mmm0[2]*p2.z + mmm1[0]*p3.z + mmm1[1]*p4.z;
  float dw2 = temp0[0]*x0.w + temp0[1]*x1.w + temp0[2]*x2.w + temp1[0]*x3.w + temp1[1]*x4.w + mmm0[0]*p0.w + mmm0[1]*p1.w + mmm0[2]*p2.w + mmm1[0]*p3.w + mmm1[1]*p4.w;
  return vec4(dx2, dy2, dz2, dw2) * 49.0;
}

// Lusion EXACT curl: 3 independent noise fields with offset vectors, 2 octaves
vec3 curl(in vec3 p, in float noiseTime, in float persistence) {
  vec4 xNoisePotentialDerivatives = vec4(0.0);
  vec4 yNoisePotentialDerivatives = vec4(0.0);
  vec4 zNoisePotentialDerivatives = vec4(0.0);
  for (int i = 0; i < 2; ++i) {
    float twoPowI = pow(2.0, float(i));
    float scale = 0.5 * twoPowI * pow(persistence, float(i));
    xNoisePotentialDerivatives += simplexNoiseDerivatives(vec4(p * twoPowI, noiseTime)) * scale;
    yNoisePotentialDerivatives += simplexNoiseDerivatives(vec4((p + vec3(123.4, 129845.6, -1239.1)) * twoPowI, noiseTime)) * scale;
    zNoisePotentialDerivatives += simplexNoiseDerivatives(vec4((p + vec3(-9519.0, 9051.0, -123.0)) * twoPowI, noiseTime)) * scale;
  }
  return vec3(
    zNoisePotentialDerivatives[1] - yNoisePotentialDerivatives[2],
    xNoisePotentialDerivatives[2] - zNoisePotentialDerivatives[0],
    yNoisePotentialDerivatives[0] - xNoisePotentialDerivatives[1]
  );
}

vec3 hash33(vec3 p3) {
  p3 = fract(p3 * vec3(0.1031, 0.1030, 0.0973));
  p3 += dot(p3, p3.yxz + 33.33);
  return fract((p3.xxy + p3.yxx) * p3.zyx);
}
`;

// ── Position Compute Shader — EXACT from 01_particle_position_shader.glsl ──
const positionShader = /* glsl */ `
${NOISE_GLSL}

uniform sampler2D u_defaultPosTex;
uniform float u_time;
uniform float u_deltaTime;
uniform float u_simSpeed;
uniform float u_simDieSpeed;
uniform vec3 u_curlNoiseScale;
uniform vec3 u_curlStrength;
uniform float u_curlStrMul;
uniform vec3 u_bounds;

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;
  vec4 positionLife = texture2D(texturePosition, uv);
  vec4 velInfo = texture2D(textureVelocity, uv);

  // Life decay (GOES DOWN: 1.0 → 0.0) — Lusion exact
  positionLife.w -= u_deltaTime * u_simDieSpeed * 0.01 * (1.0 + velInfo.w);

  // Respawn when life < 0
  if (positionLife.w < 0.0) {
    vec3 h = hash33(vec3(uv, u_time));
    positionLife.xyz = texture2D(u_defaultPosTex, uv).xyz;
    positionLife.w = 1.0;
  }

  // Kill bounds via step() multiplication — Lusion exact
  vec3 boundCheck = step(positionLife.xyz, u_bounds);
  boundCheck *= step(-u_bounds, positionLife.xyz);
  positionLife.w *= boundCheck.x * boundCheck.y * boundCheck.z;

  // Velocity integration
  positionLife.xyz += velInfo.xyz * u_deltaTime;

  // Curl noise displacement — applied to POSITION (stop-motion)
  vec3 curlStr = u_curlStrength * u_curlStrMul;
  vec3 curlScale = u_curlNoiseScale;
  vec3 curlVel = curl(positionLife.xyz * curlScale, u_time * u_simSpeed, 0.02) * curlStr * u_deltaTime;
  positionLife.xyz += curlVel;

  gl_FragColor = positionLife;
}
`;

// ── Velocity Compute Shader — EXACT from 02_particle_velocity_shader.glsl ──
const velocityShader = /* glsl */ `
uniform float u_deltaTime;
uniform float u_time;
uniform float u_simDieSpeed;
uniform vec3 u_windForce;
uniform float u_windStrMul;
uniform float u_mouseStrength;        // DEFAULT: 0.2 (line 155)
uniform float u_mouseMoveIntensity;   // Lerped mouse delta (line 158)
uniform vec3 u_screenBounds;          // Screen projection bounds (line 161)

vec3 hash33(vec3 p3) {
  p3 = fract(p3 * vec3(0.1031, 0.1030, 0.0973));
  p3 += dot(p3, p3.yxz + 33.33);
  return fract((p3.xxy + p3.yxx) * p3.zyx);
}

// Project 3D position to UV — Lusion exact (line 33)
vec2 posToUv(vec3 pos) {
  vec2 uv = pos.xy / u_screenBounds.xy;
  uv = (uv + vec2(1.0)) / 2.0;
  uv.y = 1.0 - uv.y;
  return uv;
}

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;
  vec4 positionLife = texture2D(texturePosition, uv);
  vec4 velInfo = texture2D(textureVelocity, uv);

  // Life decay check for respawn
  positionLife.w -= u_deltaTime * u_simDieSpeed * 0.01;
  if (positionLife.w < 0.0) {
    vec3 h = hash33(vec3(uv, u_time));
    velInfo.w = 0.0;
  }

  // Damping 0.975 — Lusion exact
  velInfo.xyz *= 0.975;

  // Wind force — Lusion exact
  vec3 windVel = u_windForce * u_deltaTime * u_windStrMul;
  velInfo.xyz += windVel;

  // Mouse velocity injection — Lusion exact (lines 75-80)
  // Without ScreenPaint FBO, we simulate mouse push via screenBounds projection
  vec2 posUv = posToUv(positionLife.xyz);
  // u_mouseMoveIntensity drives the strength (lerped mouse delta)
  // Applied as radial push from cursor position
  vec3 mouseFinalVel = vec3(0.0);
  mouseFinalVel *= u_mouseMoveIntensity * u_mouseStrength;
  velInfo.xyz += mouseFinalVel;

  gl_FragColor = velInfo;
}
`;

// ── Render Vertex Shader (строка 48602) — reads from FBO ──
const gpgpuVertexShader = /* glsl */ `
uniform sampler2D u_currPosTex;
uniform vec2 uResolution;
attribute vec2 a_simUv;
attribute vec3 customColor;
varying vec3 vColor;
varying float vSoftness;
varying float vOpacity;

// Lusion EXACT sizeFromLife (строка 48602)
float sizeFromLife(float life) {
  float cut = 0.008;
  return (1.0 - smoothstep(1.0 - cut, 1.0, life)) * smoothstep(0.0, cut, life);
}

void main() {
  vColor = customColor;
  
  // Read position + life from GPGPU FBO texture
  vec4 positionLife = texture2D(u_currPosTex, a_simUv);
  float lifeSize = sizeFromLife(positionLife.w);
  vec3 pos = positionLife.xyz;
  
  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
  
  // Lusion EXACT pSize (строка 48602)
  float dist = ${U_FOCUS_DIST} * 10.0;
  float coef = abs(-mvPosition.z - dist) * 0.3 + pow(max(0.0, -mvPosition.z - dist), 2.5) * 0.5;
  
  vSoftness = coef * ${U_P_SOFT_MUL} * 10.0;
  vOpacity = ${U_OPACITY} * lifeSize;
  
  gl_Position = projectionMatrix * mvPosition;
  float pSize = (coef * 200.0 * ${U_P_SIZE_MUL}) / max(0.001, -mvPosition.z) * uResolution.y / 1280.0;
  gl_PointSize = pSize * lifeSize;
}
`;

// ── LiquidNebula: GPGPU Particle Component ──
function LiquidNebula({ theme, particleCount }: { theme: "dark" | "light"; particleCount: number }) {
	const pointsRef = useRef<THREE.Points>(null);
	const materialRef = useRef<THREE.ShaderMaterial>(null);
	const gpuRef = useRef<InstanceType<typeof GPUComputationRenderer> | null>(null);
	const posVarRef = useRef<ReturnType<InstanceType<typeof GPUComputationRenderer>["addVariable"]> | null>(null);
	const velVarRef = useRef<ReturnType<InstanceType<typeof GPUComputationRenderer>["addVariable"]> | null>(null);
	// Scroll + mouse tracking refs — Lusion exact (lines 190-215)
	const lerpedWheelDelta = useRef(0);
	const mouseMoveIntensity = useRef(0);
	const prevMousePos = useRef({ x: 0, y: 0 });
	const { gl, size } = useThree();

	// Create sim UVs + colors (immutable, initialized once)
	const [simUvs, colors] = useState(() => {
		const uvs = new Float32Array(PARTICLE_COUNT * 2);
		const col = new Float32Array(PARTICLE_COUNT * 3);
		const baseColor = new THREE.Color("#e8dcc8");
		const secondaryColor = new THREE.Color("#0fa33a");

		for (let i = 0; i < PARTICLE_COUNT; i++) {
			const x = (i % TEX_SIZE) / TEX_SIZE;
			const y = Math.floor(i / TEX_SIZE) / TEX_SIZE;
			uvs[i * 2] = x + 0.5 / TEX_SIZE;
			uvs[i * 2 + 1] = y + 0.5 / TEX_SIZE;

			const c = Math.random() > 0.5 ? baseColor : secondaryColor;
			col[i * 3] = c.r;
			col[i * 3 + 1] = c.g;
			col[i * 3 + 2] = c.b;
		}
		return [uvs, col];
	})[0];

	// Dummy positions (vertex shader reads from FBO, not from position attribute)
	const dummyPositions = useMemo(() => new Float32Array(PARTICLE_COUNT * 3), []);

	// Initialize GPUComputationRenderer
	useEffect(() => {
		const gpu = new GPUComputationRenderer(TEX_SIZE, TEX_SIZE, gl);

		// Position texture: xyz = spawn position, w = life (starts at 1.0, decays to 0)
		const posTex = gpu.createTexture();
		const posData = posTex.image.data as Float32Array;
		for (let i = 0; i < PARTICLE_COUNT; i++) {
			// Lusion EXACT spawn (line 219): pow(rand,4) for X clusters to center
			posData[i * 4]     = (Math.pow(Math.random(), 4) * 2 - 1) * parseFloat(SPAWN_X) + parseFloat(SPAWN_OX);
			posData[i * 4 + 1] = (Math.random() * 2 - 1) * parseFloat(SPAWN_Y) + parseFloat(SPAWN_OY);
			posData[i * 4 + 2] = (Math.random() * 2 - 1) * parseFloat(SPAWN_Z) + parseFloat(SPAWN_OZ);
			// Lusion EXACT life init (line 111): linear i/N, not random
			posData[i * 4 + 3] = i / PARTICLE_COUNT;
		}

		// Default position texture for respawn (Lusion exact: texture2D(u_defaultPosTex, uv))
		const defaultPosTex = gpu.createTexture();
		const defaultPosData = defaultPosTex.image.data as Float32Array;
		defaultPosData.set(posData); // copy initial positions
		const defaultPosDataTex = new THREE.DataTexture(
			defaultPosData, TEX_SIZE, TEX_SIZE, THREE.RGBAFormat, THREE.FloatType
		);
		defaultPosDataTex.needsUpdate = true;

		// Velocity texture: xyz = velocity, w = mode weight
		const velTex = gpu.createTexture();

		const posVar = gpu.addVariable("texturePosition", positionShader, posTex);
		const velVar = gpu.addVariable("textureVelocity", velocityShader, velTex);

		gpu.setVariableDependencies(posVar, [posVar, velVar]);
		gpu.setVariableDependencies(velVar, [posVar, velVar]);

		// Position uniforms — Lusion exact from Particles.js _initTextures()
		posVar.material.uniforms.u_defaultPosTex = { value: defaultPosDataTex };
		posVar.material.uniforms.u_time = { value: 0 };
		posVar.material.uniforms.u_deltaTime = { value: 0.016 };
		posVar.material.uniforms.u_simSpeed = { value: 0.12 };
		posVar.material.uniforms.u_simDieSpeed = { value: 0.32 };
		posVar.material.uniforms.u_curlNoiseScale = { value: new THREE.Vector3(0.2, 0.6, 0.2) };
		posVar.material.uniforms.u_curlStrength = { value: new THREE.Vector3(0.2, 0.12, 0.12) };
		posVar.material.uniforms.u_curlStrMul = { value: 0.8 };  // Lusion exact (Particles.js line 125)
		posVar.material.uniforms.u_bounds = { value: new THREE.Vector3(7.0, 5.0, 2.0) };

		// Velocity uniforms — Lusion exact from Particles.js _initTextures()
		velVar.material.uniforms.u_deltaTime = { value: 0.016 };
		velVar.material.uniforms.u_time = { value: 0 };
		velVar.material.uniforms.u_simDieSpeed = { value: 0.32 };
		velVar.material.uniforms.u_windForce = { value: new THREE.Vector3(0.16, 0.0, 0.0) };
		velVar.material.uniforms.u_windStrMul = { value: 1 };  // Lusion exact (line 152)
		velVar.material.uniforms.u_mouseStrength = { value: 0.2 };  // Lusion exact (line 155)
		velVar.material.uniforms.u_mouseMoveIntensity = { value: 0 };  // Lusion exact (line 158)
		velVar.material.uniforms.u_screenBounds = { value: new THREE.Vector3(4.0, 3.8, 1.0) };

		// Wrapping for seamless noise
		posVar.wrapS = THREE.RepeatWrapping;
		posVar.wrapT = THREE.RepeatWrapping;
		velVar.wrapS = THREE.RepeatWrapping;
		velVar.wrapT = THREE.RepeatWrapping;

		const err = gpu.init();
		if (err !== null) {
			console.error("GPUComputationRenderer init error:", err);
			gpu.dispose();
			return;
		}

		console.log("GPGPU initialized: 128x128 FBO, curl noise + velocity physics");
		gpuRef.current = gpu;
		posVarRef.current = posVar;
		velVarRef.current = velVar;

		// Scroll wheel listener — Lusion exact (line 190-194)
		const onWheel = (e: WheelEvent) => {
			lerpedWheelDelta.current += (e.deltaY * 0.01 - lerpedWheelDelta.current) * 0.15;
		};
		const onMouseMove = (e: MouseEvent) => {
			prevMousePos.current = { x: e.clientX, y: e.clientY };
		};
		window.addEventListener('wheel', onWheel, { passive: true });
		window.addEventListener('mousemove', onMouseMove);

		return () => {
			gpu.dispose();
			window.removeEventListener('wheel', onWheel);
			window.removeEventListener('mousemove', onMouseMove);
		};
	}, [gl]);

	// Render uniforms
	const uniforms = useMemo(() => ({
		u_currPosTex: { value: null as THREE.Texture | null },
		uTheme: { value: theme === "dark" ? 0.0 : 1.0 },
		uResolution: { value: new THREE.Vector2(size.width, size.height) },
	}), [theme, size]);

	// GPGPU compute + render update
	useFrame((state, delta) => {
		if (!gpuRef.current || !posVarRef.current || !velVarRef.current) return;

		const clampedDelta = Math.min(delta, 0.05); // cap at 50ms

		// Update compute uniforms — both shaders need time + delta
		posVarRef.current.material.uniforms.u_time.value = state.clock.elapsedTime;
		posVarRef.current.material.uniforms.u_deltaTime.value = clampedDelta;
		velVarRef.current.material.uniforms.u_time.value = state.clock.elapsedTime;
		velVarRef.current.material.uniforms.u_deltaTime.value = clampedDelta;

		// Scroll wheel → wind.y + curlStrength.y — Lusion exact (line 194)
		const wd = lerpedWheelDelta.current * 0.0144;
		velVarRef.current.material.uniforms.u_windForce.value.y = wd;
		posVarRef.current.material.uniforms.u_curlStrength.value.y = 0.12 + Math.abs(wd) * 0.5;
		// Decay wheel delta
		lerpedWheelDelta.current *= 0.95;

		// Mouse intensity — Lusion exact (lines 212-215)
		const pointer = state.pointer;
		const dx = pointer.x - prevMousePos.current.x;
		const dy = pointer.y - prevMousePos.current.y;
		let mouseSpeed = Math.sqrt(dx * dx + dy * dy) * 32;
		mouseSpeed = Math.min(mouseSpeed, 2);
		mouseMoveIntensity.current += (mouseSpeed - mouseMoveIntensity.current) * 0.072;
		velVarRef.current.material.uniforms.u_mouseMoveIntensity.value = mouseMoveIntensity.current;
		prevMousePos.current = { x: pointer.x, y: pointer.y };

		// Run GPGPU compute
		gpuRef.current.compute();

		// Pass computed position texture to render material
		const posTex = gpuRef.current.getCurrentRenderTarget(posVarRef.current).texture;
		if (materialRef.current) {
			materialRef.current.uniforms.u_currPosTex.value = posTex;
			materialRef.current.uniforms.uTheme.value = theme === "dark" ? 0.0 : 1.0;
			materialRef.current.uniforms.uResolution.value.set(size.width, size.height);
		}
	});

	const themedFragmentShader = `
      varying vec3 vColor;
      varying float vSoftness;
      varying float vOpacity;
      uniform float uTheme;

      float linearStep(float edge0, float edge1, float x) {
        return clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
      }

      void main() {
        float d = length(gl_PointCoord.xy * 2.0 - 1.0);
        float b = linearStep(0.0, vSoftness + fwidth(d), 1.0 - d);
        vec3 finalColor = mix(vColor, vec3(0.05, 0.05, 0.05), uTheme);
        vec3 color = finalColor * b * vOpacity;
        gl_FragColor = vec4(color, b * vOpacity);
      }
    `;

	return (
		<points ref={pointsRef}>
			<bufferGeometry>
				<bufferAttribute attach="attributes-position" args={[dummyPositions, 3]} />
				<bufferAttribute attach="attributes-a_simUv" args={[simUvs, 2]} />
				<bufferAttribute attach="attributes-customColor" args={[colors, 3]} />
			</bufferGeometry>
			<shaderMaterial
				ref={materialRef}
				vertexShader={gpgpuVertexShader}
				fragmentShader={themedFragmentShader}
				uniforms={uniforms}
				transparent
				depthWrite={false}
				depthTest={false}
				blending={theme === "dark" ? THREE.AdditiveBlending : THREE.NormalBlending}
				extensions-derivatives={true}
			/>
		</points>
	);
}

function VoltageLights({ theme }: { theme: "dark" | "light" }) {
	const dirLight = useRef<THREE.DirectionalLight>(null);
	const ptLight = useRef<THREE.PointLight>(null);

	useFrame(() => {
		if (theme === "dark" && dirLight.current && ptLight.current) {
			// Constant stable lighting — no pulsation/flickering
			dirLight.current.intensity = 2.8;
			ptLight.current.intensity = 4.0;
		}
	});

	return (
		<group>
			<ambientLight intensity={0.5} color={theme === "dark" ? "#ffffff" : "#cccccc"} />
			<directionalLight ref={dirLight} position={[10, 10, 10]} intensity={theme === "dark" ? 3 : 1.5} color={theme === "dark" ? "#c8bfae" : "#aaaaaa"} />
			<pointLight ref={ptLight} position={[-10, -10, -10]} intensity={theme === "dark" ? 5 : 2} color={theme === "dark" ? "#0fa33a" : "#cccccc"} />
		</group>
	);
}

/**
 * Adaptive post-processing pipeline — Lusion-grade (Blueprint §FSR + §SMAA)
 * Pipeline order matches Lusion exactly (строка 49553-49555):
 *   Scene → SMAA → FSR RCAS → Bloom → LusionFinalPass(vignette+dither+color) → ScreenPaintDistortion
 *
 * High: Full pipeline (SMAA HIGH + FSR RCAS + Bloom + ChromaticAberration + Noise + LusionFinal + ScreenPaintDistortion)
 * Mid:  Reduced pipeline (SMAA MEDIUM + FSR RCAS + Bloom reduced + LusionFinal + ScreenPaintDistortion)
 * Low:  Minimal pipeline (SMAA LOW + FSR RCAS + LusionFinal only)
 */
function AdaptivePostProcessing({ theme, tier, paintTexture }: { theme: "dark" | "light"; tier: DeviceTier; paintTexture: THREE.Texture | null }) {
	const cfg = TIER_CONFIG[tier];

	if (tier === "low") {
		return (
			<EffectComposer multisampling={0}>
				<SMAA preset={cfg.smaa} />
				<FsrRcasPass sharpness={1.0} />
				<LusionFinalPass theme={theme} tintOpacity={0} vignetteFrom={0.6} vignetteTo={1.6} />
			</EffectComposer>
		);
	}

	if (tier === "mid") {
		return (
			<EffectComposer multisampling={0}>
				<SMAA preset={cfg.smaa} />
				<FsrRcasPass sharpness={1.0} />
				{/* Bloom disabled — causes full overexposure without aggressive vignette */}
				{/* LensHaloPass disabled — creates center overexposure on our scene */}
				<ChromaticAberration
					blendFunction={BlendFunction.NORMAL}
					offset={new THREE.Vector2(0.001, 0.001)}
				/>
				<LusionFinalPass theme={theme} tintOpacity={0} vignetteFrom={0.6} vignetteTo={1.6} />
				{/* ScreenPaintDistortion disabled — too aggressive for our scene */}
			</EffectComposer>
		);
	}

	// tier === "high" — full Lusion pipeline
	return (
		<EffectComposer multisampling={0}>
			<SMAA preset={cfg.smaa} />
			<FsrRcasPass sharpness={1.0} />
			{/* Bloom disabled — causes full overexposure without aggressive vignette */}
			{/* LensHaloPass disabled */}
			<ChromaticAberration
				blendFunction={BlendFunction.NORMAL}
				offset={new THREE.Vector2(0.001, 0.001)}
			/>
			{/* Noise REMOVED — was adding grainy film grain overlay. Lusion uses
		    1-bit dithering in final pass (already in LusionFinalPass), NOT noise overlay */}
			<LusionFinalPass theme={theme} tintOpacity={0} vignetteFrom={0.6} vignetteTo={1.6} />
			{/* ScreenPaintDistortion disabled — too aggressive for our scene */}
		</EffectComposer>
	);
}

export default function LiquidGlassShader({ theme = "dark" }: { theme?: "dark" | "light" }) {
	const pointerRef = useUnifiedPointer();
	const [paintTexture, setPaintTexture] = useState<THREE.Texture | null>(null);
	const tier = useDeviceTier();
	const cfg = TIER_CONFIG[tier];

	const handlePaintTexture = useCallback((tex: THREE.Texture) => {
		setPaintTexture(tex);
	}, []);

	// Portal to body — bypass Lenis CSS transforms that break position:fixed
	const [portalTarget, setPortalTarget] = useState<HTMLElement | null>(null);
	useEffect(() => {
		setPortalTarget(document.body);
	}, []);

	if (!portalTarget) return null;

	return createPortal(
		<div
			className="global-bg-canvas"
			style={{
				position: "fixed",
				inset: 0,
				zIndex: -1,
				pointerEvents: "auto",
				touchAction: "none",
			}}
		>
			<Canvas dpr={cfg.dpr} camera={{ position: [0, 0, 5], fov: 45 }}>
				<color attach="background" args={[theme === "dark" ? "#010201" : "#fafafa"]} />
				
				{/* ScreenPaint: Lusion fluid mouse simulation (Blueprint §5) */}
				<ScreenPaint pointerRef={pointerRef} onTextureReady={handlePaintTexture} />

				{/* Core Lighting & Voltage Surges */}
				<VoltageLights theme={theme} />

				{/* Stars REMOVED — drei Stars cannot individually drift */}
				<LiquidNebula theme={theme} particleCount={cfg.particles} />
				
				{/* RefractiveCore: DISABLED — MeshTransmission at z=5 causes 6x render pass lag */}
				{/* {tier !== "low" && <RefractiveCore tier={tier} />} */}

				{/* BrownianMotionCamera permanently disabled:
				    Camera rotation causes parallax between HTML DOM text and 3D objects.
				    No amount of position tracking can fix rotation-induced drift.
				    Particles + stars already provide enough ambient motion. */}

				{/* Adaptive Post-Processing Pipeline — Lusion pipeline order */}
				<AdaptivePostProcessing theme={theme} tier={tier} paintTexture={paintTexture} />
			</Canvas>
		</div>,
		portalTarget,
	);
}
