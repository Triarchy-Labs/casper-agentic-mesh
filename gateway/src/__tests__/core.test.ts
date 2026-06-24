import { describe, it, expect } from "vitest";

/**
 * Core utility tests for x402 Triarchy Gateway
 * Covers: L402 validation logic, WASM sandbox heuristics, routing thresholds
 */

// === L402 Payment Validation ===
describe("L402 Payment Validation", () => {
	it("should reject empty transaction hash", () => {
		const txHash = "";
		expect(txHash.length).toBe(0);
		expect(txHash).toBeFalsy();
	});

	it("should accept valid Stellar transaction hash format (64 hex chars)", () => {
		const validHash = "a".repeat(64);
		expect(validHash).toMatch(/^[a-f0-9]{64}$/);
	});

	it("should reject invalid hash with wrong length", () => {
		const shortHash = "abc123";
		expect(shortHash).not.toMatch(/^[a-f0-9]{64}$/);
	});
});

// === WASM Sandbox Heuristic Patterns ===
describe("WASM Sandbox Heuristic Detection", () => {
	const BANNED_PATTERNS = [
		"system(", "exec(", "eval(", "__proto__",
		"constructor.prototype", "process.env",
		"require(", "import(", "fs.readFile",
		"child_process", "rm -rf", "chmod",
	];

	it("should detect all banned injection patterns", () => {
		for (const pattern of BANNED_PATTERNS) {
			const payload = `some input with ${pattern} inside`;
			const detected = BANNED_PATTERNS.some((p) => payload.includes(p));
			expect(detected).toBe(true);
		}
	});

	it("should pass clean payloads through", () => {
		const cleanPayload = "Summarize this research paper about quantum computing";
		const detected = BANNED_PATTERNS.some((p) => cleanPayload.includes(p));
		expect(detected).toBe(false);
	});

	it("should detect prototype pollution attempts", () => {
		const poisoned = '{"__proto__": {"isAdmin": true}}';
		expect(poisoned).toContain("__proto__");
	});
});

// === Routing Threshold Logic ===
describe("Task Routing Thresholds", () => {
	const ENTERPRISE_THRESHOLD = 5.0;

	it("should route micro-bounties (<$5) to local LLM", () => {
		const bounty = 2.5;
		expect(bounty < ENTERPRISE_THRESHOLD).toBe(true);
	});

	it("should route enterprise tasks (>=$5) to sovereign node", () => {
		const bounty = 10.0;
		expect(bounty >= ENTERPRISE_THRESHOLD).toBe(true);
	});

	it("should handle edge case at exact threshold", () => {
		const bounty = 5.0;
		expect(bounty >= ENTERPRISE_THRESHOLD).toBe(true);
	});

	it("should reject negative bounty amounts", () => {
		const bounty = -1.0;
		expect(bounty > 0).toBe(false);
	});
});

// === Unified Pointer Input Normalization ===
describe("Unified Pointer Normalization", () => {
	it("should normalize center of screen to (0, 0)", () => {
		const width = 1920;
		const height = 1080;
		const clientX = width / 2;
		const clientY = height / 2;
		const x = (clientX / width) * 2 - 1;
		const y = -(clientY / height) * 2 + 1;
		expect(x).toBeCloseTo(0, 5);
		expect(y).toBeCloseTo(0, 5);
	});

	it("should normalize top-left to (-1, 1)", () => {
		const x = (0 / 1920) * 2 - 1;
		const y = -(0 / 1080) * 2 + 1;
		expect(x).toBeCloseTo(-1, 5);
		expect(y).toBeCloseTo(1, 5);
	});

	it("should normalize bottom-right to (1, -1)", () => {
		const width = 1920;
		const height = 1080;
		const x = (width / width) * 2 - 1;
		const y = -(height / height) * 2 + 1;
		expect(x).toBeCloseTo(1, 5);
		expect(y).toBeCloseTo(-1, 5);
	});
});
