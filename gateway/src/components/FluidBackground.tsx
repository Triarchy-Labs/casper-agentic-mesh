"use client";
import React, { useRef, useEffect } from "react";
import * as THREE from "three";

export default function FluidBackground() {
  const mountRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!mountRef.current) return;

    // Scene setup
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x000000);
    scene.fog = new THREE.FogExp2(0x000000, 0.08);

    const camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );
    camera.position.z = 5;

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    mountRef.current.appendChild(renderer.domElement);

    // Geometry - High resolution plane for fluid tissue effect
    const geometry = new THREE.PlaneGeometry(20, 20, 128, 128);

    // Custom Shader for Painted Texture reacting to cursor
    const material = new THREE.ShaderMaterial({
      uniforms: {
        uTime: { value: 0.0 },
        uMouse: { value: new THREE.Vector2(0.5, 0.5) },
        uColorPrimary: { value: new THREE.Color(0x111111) }, // Vercel Dark gray
        uColorSecondary: { value: new THREE.Color(0xff0000) }, // Casper Red
      },
      vertexShader: `
        uniform float uTime;
        uniform vec2 uMouse;
        varying vec2 vUv;
        varying float vElevation;

        // Simplex noise function (pseudo)
        vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
        vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }
        float snoise(vec2 v) {
          const vec4 C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
          vec2 i  = floor(v + dot(v, C.yy));
          vec2 x0 = v -   i + dot(i, C.xx);
          vec2 i1;
          i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
          vec4 x12 = x0.xyxy + C.xxzz;
          x12.xy -= i1;
          i = mod289(i);
          vec3 p = permute(permute( i.y + vec3(0.0, i1.y, 1.0 )) + i.x + vec3(0.0, i1.x, 1.0 ));
          vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
          m = m*m;
          m = m*m;
          vec3 x = 2.0 * fract(p * C.www) - 1.0;
          vec3 h = abs(x) - 0.5;
          vec3 ox = floor(x + 0.5);
          vec3 a0 = x - ox;
          m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );
          vec3 g;
          g.x  = a0.x  * x0.x  + h.x  * x0.y;
          g.yz = a0.yz * x12.xz + h.yz * x12.yw;
          return 130.0 * dot(m, g);
        }

        void main() {
          vUv = uv;
          
          // Base noise
          float noise = snoise(vec2(position.x * 0.5 + uTime * 0.1, position.y * 0.5 + uTime * 0.1));
          
          // Mouse interaction wave
          float dist = distance(uv, uMouse);
          float mouseWave = sin(dist * 10.0 - uTime * 2.0) * exp(-dist * 5.0);
          
          vElevation = noise * 0.5 + mouseWave * 0.8;
          
          vec3 newPosition = position;
          newPosition.z += vElevation;
          
          gl_Position = projectionMatrix * modelViewMatrix * vec4(newPosition, 1.0);
        }
      `,
      fragmentShader: `
        uniform vec3 uColorPrimary;
        uniform vec3 uColorSecondary;
        varying float vElevation;
        varying vec2 vUv;

        void main() {
          // Map elevation to color mixing
          float mixStrength = (vElevation + 0.5) * 0.5;
          
          // Subtle painted effect
          vec3 color = mix(uColorPrimary, uColorSecondary, mixStrength * 0.3);
          
          // Add some vignette/depth
          float vignette = distance(vUv, vec2(0.5));
          color *= 1.0 - vignette * 0.5;
          
          gl_FragColor = vec4(color, 1.0);
        }
      `,
      wireframe: false,
    });

    const mesh = new THREE.Mesh(geometry, material);
    // Tilt the plane slightly to look like a surface extending backwards
    mesh.rotation.x = -Math.PI * 0.15;
    scene.add(mesh);

    // Mouse Tracking
    const mouse = new THREE.Vector2(0.5, 0.5);
    const targetMouse = new THREE.Vector2(0.5, 0.5);

    const onMouseMove = (event: MouseEvent) => {
      targetMouse.x = event.clientX / window.innerWidth;
      targetMouse.y = 1.0 - (event.clientY / window.innerHeight);
    };
    window.addEventListener("mousemove", onMouseMove);

    // Resize Handler
    const onResize = () => {
      camera.aspect = window.innerWidth / window.innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(window.innerWidth, window.innerHeight);
    };
    window.addEventListener("resize", onResize);

    // Animation Loop
    const clock = new THREE.Clock();
    let animationId: number;

    const tick = () => {
      const elapsedTime = clock.getElapsedTime();
      
      // Smooth mouse interpolation (fluid reaction)
      mouse.x += (targetMouse.x - mouse.x) * 0.05;
      mouse.y += (targetMouse.y - mouse.y) * 0.05;

      material.uniforms.uTime.value = elapsedTime;
      material.uniforms.uMouse.value.set(mouse.x, mouse.y);

      renderer.render(scene, camera);
      animationId = window.requestAnimationFrame(tick);
    };

    tick();

    // Cleanup
    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("resize", onResize);
      window.cancelAnimationFrame(animationId);
      if (mountRef.current && renderer.domElement) {
        mountRef.current.removeChild(renderer.domElement);
      }
      geometry.dispose();
      material.dispose();
      renderer.dispose();
    };
  }, []);

  return (
    <div
      ref={mountRef}
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        zIndex: 0, // Base layer
        pointerEvents: "none", // Let clicks pass through to content
      }}
    />
  );
}
