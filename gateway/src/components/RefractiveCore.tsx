"use client";
import { Icosahedron, MeshTransmissionMaterial } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import { useRef } from "react";
import type * as THREE from "three";
import type { DeviceTier } from "../hooks/useDeviceTier";

/**
 * RefractiveCore — Glass icosahedron with transmission material
 * 
 * Performance note: MeshTransmissionMaterial re-renders the ENTIRE scene
 * into a secondary FBO `resolution × resolution` pixels, `samples` times per frame.
 * At resolution=1024, samples=5, that's 5 full scene re-draws per frame.
 * 
 * Lusion approach: They don't use transmission material at all — they use
 * custom shaders. We adapt by reducing resolution/samples on weaker devices.
 * 
 * On "low" tier, this component is not rendered at all (handled by parent).
 */

// Adaptive config per tier
const TIER_CONFIG = {
	high: { resolution: 512, samples: 3, chromaticAberration: 2, distortion: 0.8 },
	mid:  { resolution: 256, samples: 2, chromaticAberration: 1, distortion: 0.5 },
	low:  { resolution: 128, samples: 1, chromaticAberration: 0.5, distortion: 0.3 },
};

export default function RefractiveCore({ tier = "high" }: { tier?: DeviceTier }) {
	const meshRef = useRef<THREE.Mesh>(null);
	const cfg = TIER_CONFIG[tier];

	useFrame((state) => {
		if (meshRef.current) {
			meshRef.current.rotation.x = state.clock.elapsedTime * 0.15;
			meshRef.current.rotation.y = state.clock.elapsedTime * 0.25;
		}
	});

	return (
		<Icosahedron ref={meshRef} args={[1.2, 4]} position={[0, 0.5, 0]}>
			<MeshTransmissionMaterial
				backside
				samples={cfg.samples}
				thickness={3.5}
				chromaticAberration={cfg.chromaticAberration}
				anisotropy={0.5}
				distortion={cfg.distortion}
				distortionScale={0.5}
				temporalDistortion={0.3}
				transmission={1.0}
				roughness={0.05}
				ior={1.6}
				color="rgba(255,255,255,0.85)"
				resolution={cfg.resolution}
			/>
		</Icosahedron>
	);
}
