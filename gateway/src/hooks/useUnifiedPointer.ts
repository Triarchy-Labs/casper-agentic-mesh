"use client";
import { useCallback, useEffect, useRef } from "react";

/**
 * Unified Input — Lusion-pattern singleton pointer normalizer.
 * Normalizes all touch/mouse events into [-1, 1] coordinate space
 * with velocity (dx, dy) for ScreenPaint fluid simulation.
 */

export interface PointerState {
	/** Normalized X [-1 (left), 1 (right)] */
	x: number;
	/** Normalized Y [-1 (bottom), 1 (top)] */
	y: number;
	/** Frame delta X */
	dx: number;
	/** Frame delta Y */
	dy: number;
	/** Pixel X (raw) */
	px: number;
	/** Pixel Y (raw) */
	py: number;
	/** Is pointer down */
	isDown: boolean;
	/** Is touch device */
	isTouch: boolean;
}

const initialState: PointerState = {
	x: 0, y: 0, dx: 0, dy: 0,
	px: 0, py: 0,
	isDown: false, isTouch: false,
};

/**
 * Returns a mutable ref to the latest pointer state (no re-renders).
 * Used by ScreenPaint and CameraControls to read pointer data each frame.
 */
export function useUnifiedPointer() {
	const state = useRef<PointerState>({ ...initialState });
	const prevPx = useRef(0);
	const prevPy = useRef(0);

	const update = useCallback((clientX: number, clientY: number) => {
		const s = state.current;
		s.px = clientX;
		s.py = clientY;
		s.x = (clientX / window.innerWidth) * 2 - 1;
		s.y = -(clientY / window.innerHeight) * 2 + 1;
		s.dx = clientX - prevPx.current;
		s.dy = clientY - prevPy.current;
		prevPx.current = clientX;
		prevPy.current = clientY;
	}, []);

	useEffect(() => {
		const onPointerMove = (e: PointerEvent) => {
			update(e.clientX, e.clientY);
		};
		const onPointerDown = (e: PointerEvent) => {
			state.current.isDown = true;
			state.current.isTouch = e.pointerType === "touch";
			update(e.clientX, e.clientY);
		};
		const onPointerUp = () => {
			state.current.isDown = false;
			// Zero delta on release to stop ScreenPaint trail
			state.current.dx = 0;
			state.current.dy = 0;
		};

		window.addEventListener("pointermove", onPointerMove, { passive: true });
		window.addEventListener("pointerdown", onPointerDown, { passive: true });
		window.addEventListener("pointerup", onPointerUp, { passive: true });
		window.addEventListener("pointercancel", onPointerUp, { passive: true });

		return () => {
			window.removeEventListener("pointermove", onPointerMove);
			window.removeEventListener("pointerdown", onPointerDown);
			window.removeEventListener("pointerup", onPointerUp);
			window.removeEventListener("pointercancel", onPointerUp);
		};
	}, [update]);

	return state;
}
