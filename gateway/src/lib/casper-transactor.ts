import { validateCasperPayment } from "./casper";
import { validateForeignPayload } from "./wasm_sandbox";
import * as fs from "node:fs";

/**
 * CASPER TRANSACTOR PIPELINE
 * 
 * Ensures secure transaction verification and execution for the x402 Gateway.
 * 
 * 1. preflight() - Cheap syntax, signature, and format checks. NO external RPCs.
 * 2. preclaim() - Expensive checks. Casper RPC calls. Ledger state checks. WASM Sandbox.
 * 3. checkQueueState() - Modifies pipeline state to evaluate execution queue limits.
 */

export interface TransactorContext {
    txHash: string;
    bountyUsdc: number;
    description: string;
    clientId: string;
    taskId: string;
}

export class CasperTransactor {
    /**
     * @brief Checks structural validity of the payload.
     * Rejects early before making expensive Casper RPC calls.
     */
    static preflight(ctx: TransactorContext): { valid: boolean; error?: string } {
        if (!ctx.txHash) {
            return { valid: false, error: "Payment Required. Please provide x-l402-txhash header." };
        }
        if (!ctx.description || ctx.bountyUsdc === undefined || ctx.bountyUsdc < 0) {
            return { valid: false, error: "Malformed payload: missing definition or negative bounty." };
        }
        // Strict boundary checks
        if (ctx.bountyUsdc > 1000000) {
            return { valid: false, error: "Bounty exceeds maximum gateway tier." };
        }
        return { valid: true };
    }

    /**
     * @brief Validates state and conditions (Casper RPC + WASM Sandbox).
     */
    static async preclaim(ctx: TransactorContext): Promise<{ valid: boolean; error?: string; refundedUsdc?: number; details?: string }> {
        const expectedMemo = ctx.clientId || ctx.taskId || "demo";
        
        // 1. Casper Validation (Testnet validation)
        const validation = await validateCasperPayment(ctx.txHash, ctx.bountyUsdc, expectedMemo);
        if (!validation.valid) {
            return { valid: false, error: `x402 Payment Validation Failed: ${validation.error}` };
        }

        // 2. WASI 0.2 Payload Audit (Malicious execution firewall)
        const pPayload = JSON.stringify({ instruction: ctx.description, origin: ctx.clientId });
        const sandboxResult = await validateForeignPayload(pPayload);

        if (!sandboxResult.safe) {
            console.warn(`[OPSEC FIREWALL] Blocked malicious payload from ${ctx.clientId}. Refunded.`);
            return { 
                valid: false, 
                error: "Payload blocked by WASI Sandbox (Malicious Payload Detected)", 
                refundedUsdc: ctx.bountyUsdc, 
                details: sandboxResult.error 
            };
        }

        return { valid: true };
    }

    /**
     * @brief Modifies pipeline state to evaluate Queue limits
     */
    static checkQueueState(localExecutionHook?: string): boolean {
        let queueLength = 0;
        if (localExecutionHook && fs.existsSync(localExecutionHook)) {
            const contents = fs.readFileSync(localExecutionHook, "utf-8");
            queueLength = contents.split("\n").filter(l => l.trim()).length;
        }
        return queueLength >= 10;
    }
}
