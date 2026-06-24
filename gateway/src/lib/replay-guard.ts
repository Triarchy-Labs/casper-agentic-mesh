/**
 * REPLAY GUARD — Anti-Replay Protection for x402 Payment Signatures
 * 
 * Assimilated from: Toll (5th Place Casper Hacks)
 * Source: toll/packages/gateway/src/middleware.ts:36-55
 * 
 * Prevents double-spending by tracking used payment signatures with TTL.
 * Without this, a single txHash can be reused indefinitely.
 */

const REPLAY_TTL_MS = 5 * 60 * 1000; // 5 minutes

export class ReplayGuard {
    private used = new Map<string, number>();

    /** Check if signature was already used. Returns true if REPLAY detected. */
    check(signature: string): boolean {
        this.cleanup();
        return this.used.has(signature);
    }

    /** Mark a signature as used after successful payment verification. */
    mark(signature: string): void {
        this.used.set(signature, Date.now());
    }

    /** Get current number of tracked signatures (for monitoring). */
    get size(): number {
        return this.used.size;
    }

    /** Remove expired entries to prevent memory leak. */
    private cleanup(): void {
        const cutoff = Date.now() - REPLAY_TTL_MS;
        for (const [sig, ts] of this.used) {
            if (ts < cutoff) this.used.delete(sig);
        }
    }
}

// Singleton instance for the gateway
export const replayGuard = new ReplayGuard();
