/**
 * SPENDING POLICY — Per-Caller Budget Enforcement
 * 
 * Assimilated from: Toll (5th Place Stellar Hacks)
 * Source: toll/packages/gateway/src/spendingPolicy.ts
 * 
 * Enforces budget caps and access controls BEFORE payment verification.
 * Policies checked in order:
 *   1. Caller allowlist/blocklist
 *   2. Per-call maximum (rejects overly expensive calls)
 *   3. Daily budget cap per caller
 *   4. Global daily budget cap across all callers
 */

export interface SpendingPolicyConfig {
    /** Max USDC a single call can cost. Rejects tasks priced above this. */
    maxPerCall?: number;
    /** Max USDC a single caller can spend per day */
    maxDailyPerCaller?: number;
    /** Max USDC total across all callers per day */
    maxDailyGlobal?: number;
    /** Stellar addresses allowed to submit tasks (empty = allow all) */
    allowedCallers?: string[];
    /** Stellar addresses blocked from submitting tasks */
    blockedCallers?: string[];
}

interface DailySpend {
    amount: number;
    dayStart: number;
}

export class SpendingPolicy {
    private config: SpendingPolicyConfig;
    private perCallerSpend = new Map<string, DailySpend>();
    private globalSpend: DailySpend = { amount: 0, dayStart: this.todayStart() };

    constructor(config: SpendingPolicyConfig = {}) {
        this.config = config;
    }

    /**
     * Check if a call is allowed under current spending policies.
     * Returns null if allowed, or an error string if rejected.
     */
    check(callerId: string, price: number): string | null {
        if (price === 0) return null;

        // 1. Caller allowlist
        if (this.config.allowedCallers?.length) {
            if (!this.config.allowedCallers.includes(callerId)) {
                return `Caller ${callerId.substring(0, 8)}... not in allowlist`;
            }
        }

        // 2. Caller blocklist
        if (this.config.blockedCallers?.includes(callerId)) {
            return `Caller ${callerId.substring(0, 8)}... is blocked`;
        }

        // 3. Per-call maximum
        if (this.config.maxPerCall !== undefined) {
            if (price > this.config.maxPerCall) {
                return `Task price $${price} exceeds per-call limit of $${this.config.maxPerCall}`;
            }
        }

        // 4. Daily per-caller budget
        if (this.config.maxDailyPerCaller !== undefined) {
            const spent = this.getCallerSpend(callerId);
            if (spent + price > this.config.maxDailyPerCaller) {
                return `Caller daily budget exhausted ($${spent.toFixed(2)} of $${this.config.maxDailyPerCaller} used)`;
            }
        }

        // 5. Global daily budget
        if (this.config.maxDailyGlobal !== undefined) {
            const spent = this.getGlobalSpend();
            if (spent + price > this.config.maxDailyGlobal) {
                return `Global daily budget exhausted ($${spent.toFixed(2)} of $${this.config.maxDailyGlobal} used)`;
            }
        }

        return null;
    }

    /** Record a successful spend */
    record(callerId: string, amount: number): void {
        const today = this.todayStart();

        // Per-caller
        const callerEntry = this.perCallerSpend.get(callerId);
        if (!callerEntry || callerEntry.dayStart !== today) {
            this.perCallerSpend.set(callerId, { amount, dayStart: today });
        } else {
            callerEntry.amount += amount;
        }

        // Global
        if (this.globalSpend.dayStart !== today) {
            this.globalSpend = { amount, dayStart: today };
        } else {
            this.globalSpend.amount += amount;
        }
    }

    getCallerSpend(callerId: string): number {
        const entry = this.perCallerSpend.get(callerId);
        if (!entry || entry.dayStart !== this.todayStart()) return 0;
        return entry.amount;
    }

    getGlobalSpend(): number {
        if (this.globalSpend.dayStart !== this.todayStart()) return 0;
        return this.globalSpend.amount;
    }

    private todayStart(): number {
        const d = new Date();
        d.setHours(0, 0, 0, 0);
        return d.getTime();
    }
}

// Default production policy
export const spendingPolicy = new SpendingPolicy({
    maxPerCall: 10000,        // Max $10,000 per single task
    maxDailyPerCaller: 50000, // Max $50,000 per caller per day
    maxDailyGlobal: 500000,   // Max $500,000 global per day
});
