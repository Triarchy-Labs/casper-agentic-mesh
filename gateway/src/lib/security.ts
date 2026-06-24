/**
 * SSRF PROTECTION — URL Allowlist Filter
 * 
 * Assimilated from: RenderGate (3rd Place Casper Hacks)
 * Source: rendergate/server.js:22-38
 * 
 * Blocks requests to internal/private networks to prevent SSRF attacks.
 * Must be called before any external fetch() in the gateway pipeline.
 */

const BLOCKED_HOSTNAMES = new Set([
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "[::1]",
    "metadata.google.internal",
    "169.254.169.254",
]);

/**
 * Validates that a URL points to a public, external resource.
 * Returns true if the URL is safe to fetch. Returns false for:
 * - Non-http(s) protocols
 * - localhost, loopback, link-local addresses
 * - Private subnets (10.x, 172.16-31.x, 192.168.x)
 * - IPv6 literals
 */
export function isAllowedUrl(urlStr: string): boolean {
    try {
        const parsed = new URL(urlStr);

        // Only allow http/https
        if (!["http:", "https:"].includes(parsed.protocol)) return false;

        // Block known dangerous hostnames
        if (BLOCKED_HOSTNAMES.has(parsed.hostname)) return false;

        // Block all IPv6 literals
        if (parsed.hostname.startsWith("[")) return false;

        // Block private IPv4 subnets
        const parts = parsed.hostname.split(".");
        if (parts.length === 4 && parts.every(p => /^\d+$/.test(p))) {
            const [a, b] = parts.map(Number);
            if (a === 10) return false;                          // 10.0.0.0/8
            if (a === 172 && b >= 16 && b <= 31) return false;   // 172.16.0.0/12
            if (a === 192 && b === 168) return false;            // 192.168.0.0/16
            if (a === 169 && b === 254) return false;            // Link-local
        }

        return true;
    } catch {
        return false;
    }
}

/**
 * BUDGET DEGRADATION — Graceful Degradation When Budget Exhausted
 * 
 * Assimilated from: Toll (5th Place Casper Hacks)
 * Source: toll/packages/gateway/src/budgetDegradation.ts
 * 
 * Three strategies:
 *   - reject: flat rejection
 *   - downgrade: suggest a cheaper alternative tool
 *   - queue: queue the request with a TTL
 */

export type DegradationStrategy = "reject" | "downgrade" | "queue";

export interface DegradationConfig {
    strategy?: DegradationStrategy;
    /** Map of premium_tool -> basic_tool for downgrade path */
    downgradeMap?: Record<string, string>;
    queueTtlSeconds?: number;
}

export interface DegradationResult {
    action: DegradationStrategy;
    alternativeTool?: string;
    queueId?: string;
    message: string;
}

export class BudgetDegradation {
    private config: DegradationConfig;

    constructor(config: DegradationConfig = { strategy: "reject" }) {
        this.config = config;
    }

    handle(toolName: string): DegradationResult {
        const strategy = this.config.strategy ?? "reject";

        if (strategy === "downgrade") {
            const alternative = this.config.downgradeMap?.[toolName];
            if (alternative) {
                return {
                    action: "downgrade",
                    alternativeTool: alternative,
                    message: `Budget exhausted. Downgrading to '${alternative}' (cheaper alternative).`,
                };
            }
        }

        if (strategy === "queue") {
            const queueId = `q-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
            return {
                action: "queue",
                queueId,
                message: `Budget exhausted. Request queued (ID: ${queueId}). Top up and retry.`,
            };
        }

        return {
            action: "reject",
            message: "Budget exhausted. Payment required to continue.",
        };
    }
}
