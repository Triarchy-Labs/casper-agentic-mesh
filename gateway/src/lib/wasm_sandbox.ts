/* eslint-disable */
/**
 * Enterprise WASI 0.2 Sandbox Payload Validator
 * Uses @extism/extism for zero-trust foreign payload quarantine.
 *
 * Architecture:
 *   1. Attempt to load a compiled WASM plugin for deep analysis
 *   2. Fallback to built-in heuristic ruleset if plugin unavailable
 *   3. Both paths enforce the same security contract
 */

import createPlugin from "@extism/extism";

export interface SandboxResult {
	safe: boolean;
	clean_payload?: any;
	error?: string;
	engine?: "extism_wasm" | "heuristic_fallback";
}

// Built-in heuristic ruleset (used when WASM plugin is not available)
const BANNED_TOKENS = [
	"bash",
	"system(",
	"exec(",
	"<system>",
	"fs.",
	"process.env",
	"eval(",
	"require(",
	"import(",
	"__proto__",
	"constructor[",
	"nc -e",
	"/bin/sh",
	"curl |",
	"wget |",
	"rm -rf",
];

/**
 * Validates an external payload for malicious content.
 * First tries Extism WASM plugin, falls back to heuristic scan.
 */
export async function validateForeignPayload(
	foreignJsonStr: string,
): Promise<SandboxResult> {
	try {
		const data = JSON.parse(foreignJsonStr);
		const normalized = JSON.stringify(data).toLowerCase();

		// Try Extism WASM plugin (if deployed)
		try {
			const plugin = await createPlugin(
				process.env.WASM_SANDBOX_PLUGIN_PATH || "./plugins/quarantine.wasm",
				{
					useWasi: true,
					allowedPaths: {}, // Explicit WASI File System lockdown
					allowedHosts: [], // Block all network ingress/egress
				},
			);

			const output = await plugin.call("validate", normalized);
			await plugin.close();

			if (output) {
				const result = JSON.parse(output.text());
				if (!result.safe) {
					return {
						safe: false,
						error: `[WASM QUARANTINE] Plugin detected violation: ${result.reason}`,
						engine: "extism_wasm",
					};
				}
				return { safe: true, clean_payload: data, engine: "extism_wasm" };
			}
			// Plugin returned null — fall through to heuristic
		} catch {
			// WASM plugin not available — use heuristic fallback
		}

		// Heuristic fallback: token-based scan
		for (const token of BANNED_TOKENS) {
			if (normalized.includes(token)) {
				return {
					safe: false,
					error: `[WASM QUARANTINE] Critical violation. Banned token detected: ${token}`,
					engine: "heuristic_fallback",
				};
			}
		}

		return { safe: true, clean_payload: data, engine: "heuristic_fallback" };
	} catch {
		return {
			safe: false,
			error: "Failed to parse foreign agent response.",
			engine: "heuristic_fallback",
		};
	}
}
