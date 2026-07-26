import { VerifyOptions, X402VerificationResult } from "../types/index.js";

const DEFAULT_RPC = "https://rpc.testnet.casperlabs.io/rpc";

/**
 * Normalizes an Account Hash into a standard 64-character lowercase hex string.
 * This edge-compatible version does NOT do PublicKey-to-AccountHash (Blake2b) conversion.
 * L402 clients are expected to send native Transfers using Account Hash targets.
 */
export function canonicalizeAccountHash(identifier: string): string {
    let clean = identifier.toLowerCase().trim();
    if (clean.startsWith('account-hash-')) {
        clean = clean.replace('account-hash-', '');
    }
    
    if (!/^[0-9a-f]{64}$/.test(clean)) {
        throw new Error(`Invalid Account Hash format: ${clean}`);
    }

    return clean;
}

export async function verifyTransactionOnChain(
    txHash: string,
    options: VerifyOptions
): Promise<X402VerificationResult> {
    try {
        const rpcUrl = options.rpcUrl || DEFAULT_RPC;
        
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 10000);

        // Native Edge-Compatible Fetch to Casper RPC
        let response: Response;
        try {
            response = await fetch(rpcUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: "2.0",
                    id: 1,
                    method: "info_get_deploy",
                    params: { deploy_hash: txHash }
                }),
                signal: controller.signal,
            });
        } finally {
            clearTimeout(timeoutId);
        }

        if (!response.ok) {
            throw new Error(`RPC HTTP Error: ${response.status}`);
        }

        const rpcData = await response.json();
        
        if (rpcData.error) {
            return { authorized: false, reason: `RPC Error: ${rpcData.error.message}` };
        }

        const deployResult = rpcData.result;
        if (!deployResult || !deployResult.deploy) {
            return { authorized: false, reason: "Transaction not found on chain." };
        }

        const deploy = deployResult.deploy;
        const executionResults = deployResult.execution_results;

        // 2. Validate successful execution
        if (!executionResults || executionResults.length === 0) {
            return { authorized: false, reason: "Transaction is still pending in mempool. Please wait for the block to finalize." };
        }
        
        const result = executionResults[0].result;
        if (result.Failure) {
            return { authorized: false, reason: `Transaction failed during execution on-chain: ${result.Failure.error_message}` };
        }
        if (!result.Success) {
            return { authorized: false, reason: "Transaction execution status unknown." };
        }

        // 3. Replay Protection - Tx Age Validation
        const txTimestamp = new Date(deploy.header.timestamp).getTime();
        const maxAgeMs = (options.maxAgeSeconds || 3600) * 1000;
        if (Date.now() - txTimestamp > maxAgeMs) {
            return { authorized: false, reason: `Transaction expired. Maximum allowed age is ${maxAgeMs / 1000} seconds.` };
        }

        // 4. Replay Protection - State Store Validation
        if (options.replayStore) {
            const consumed = await options.replayStore.has(txHash);
            if (consumed) {
                return { authorized: false, reason: "L402 Payment Required: Transaction hash already consumed (Replay Attack detected)." };
            }
        }

        // 5. Validate Chain Name if specified
        if (options.expectedChainName && deploy.header.chain_name !== options.expectedChainName) {
            return { authorized: false, reason: `Invalid chain network. Expected ${options.expectedChainName}` };
        }

        // 6. Deep Argument Parsing (Transfer Session)
        const session = deploy.session;
        if (!session.Transfer) {
            return { authorized: false, reason: "Transaction is not a native CSPR transfer." };
        }

        const args = session.Transfer.args;
        let amountMotes: bigint | null = null;
        let targetAccountHash: string | null = null;
        let paymentId: string | undefined;

        for (const arg of args) {
            const argName = arg[0];
            const argData = arg[1];
            
            if (argName === "amount") {
                amountMotes = BigInt(argData.parsed);
            }
            if (argName === "target") {
                // If it's ByteArray (AccountHash), bytes is the raw hex. 
                // If it's PublicKey, the user should be rejected unless it matches Account Hash.
                if (argData.cl_type && argData.cl_type.ByteArray === 32) {
                    targetAccountHash = argData.bytes;
                } else if (argData.parsed) {
                    targetAccountHash = typeof argData.parsed === 'string' ? argData.parsed : argData.bytes;
                } else {
                    targetAccountHash = argData.bytes;
                }
            }
            if (argName === "id") {
                paymentId = argData.parsed !== null ? String(argData.parsed) : undefined;
            }
        }

        if (amountMotes === null || !targetAccountHash) {
            return { authorized: false, reason: "Invalid transfer arguments (missing amount or target)." };
        }

        // Anti-Hijacking Check (Front-running protection)
        if (options.expectedPaymentId && paymentId !== options.expectedPaymentId.toString()) {
            return { authorized: false, reason: `Payment ID mismatch. Expected ${options.expectedPaymentId}, found ${paymentId || 'None'}.` };
        }

        // 7. Recipient Verification
        // Clean target bytes. A ByteArray serialization in Casper usually doesn't have prefixes, but let's be safe.
        const canonicalTarget = canonicalizeAccountHash(targetAccountHash);
        const canonicalRequired = canonicalizeAccountHash(options.requiredGatewayAccount);

        if (canonicalTarget !== canonicalRequired) {
            return { 
                authorized: false, 
                reason: `Payment sent to wrong recipient. Expected ${options.requiredGatewayAccount}, but got ${canonicalTarget}` 
            };
        }

        // 8. Amount Verification
        const requiredMotes = BigInt(options.minimumPaymentMotes.toString());
        if (amountMotes < requiredMotes) {
            return { 
                authorized: false, 
                reason: `Insufficient payment. Received ${amountMotes} motes, required ${requiredMotes} motes.` 
            };
        }

        // 9. Mark transaction as consumed in Replay Store
        if (options.replayStore) {
            await options.replayStore.add(txHash, maxAgeMs);
        }

        return {
            authorized: true,
            payerPublicKey: deploy.header.account,
            transferredMotes: amountMotes.toString(),
            paymentId,
            timestamp: deploy.header.timestamp
        };

    } catch (error: any) {
        return { authorized: false, reason: `Verification error: ${error.message}` };
    }
}
