export interface X402VerificationResult {
    authorized: boolean;
    reason?: string;
    payerAccountHash?: string;
    payerPublicKey?: string;
    transferredMotes?: string;
    timestamp?: string;
    paymentId?: string | number;
}

export interface IReplayStore {
    /** Checks if transaction hash has already been consumed */
    has(txHash: string): Promise<boolean>;
    /** Marks transaction hash as consumed for a specific TTL window (in ms) */
    add(txHash: string, ttlMs?: number): Promise<void>;
}

export interface VerifyOptions {
    /** Target account hash or public key expected to receive funds */
    requiredGatewayAccount: string;
    /** Minimum payment required in motes (1 CSPR = 1,000,000,000 motes) */
    minimumPaymentMotes: bigint | number | string;
    /** Optional expected network name (e.g. 'casper-test' or 'casper') */
    expectedChainName?: string;
    /** Expected payment ID / nonce to prevent front-running hijacking */
    expectedPaymentId?: string | number;
    /** Maximum allowed transaction age in seconds to prevent stale tx reuse (default: 3600s) */
    maxAgeSeconds?: number;
    /** Stateful replay store to prevent double-spending the same txHash */
    replayStore?: IReplayStore;
    /** Custom Casper JSON-RPC URL */
    rpcUrl?: string;
}
