import { NextResponse } from 'next/server';
import { verifyTransactionOnChain } from "../services/casper.js";
import { VerifyOptions } from "../types/index.js";

/**
 * Next.js Edge-compatible middleware for protecting routes with L402 payments.
 * 
 * @param options - Configuration for verification (gateway account, minimum motes, etc.)
 * @returns A Next.js middleware function handler
 */
export function nextX402Middleware(options: VerifyOptions) {
    return async (req: Request) => {
        const authHeader = req.headers.get("authorization");

        if (!authHeader || !authHeader.startsWith("L402 ")) {
            return new Response(JSON.stringify({ error: "L402 Payment Required" }), {
                status: 402,
                headers: {
                    "Content-Type": "application/json",
                    "WWW-Authenticate": `L402 target="${options.requiredGatewayAccount}", min_motes="${options.minimumPaymentMotes}"`
                }
            });
        }

        const txHash = authHeader.split(" ")[1];

        try {
            const verification = await verifyTransactionOnChain(txHash, options);

            if (verification.authorized) {
                const response = NextResponse.next();
                response.headers.set('x-x402-payer', verification.payerPublicKey || '');
                response.headers.set('x-x402-payment-id', verification.paymentId?.toString() || '');
                return response;
            }

            return new Response(JSON.stringify({ error: verification.reason }), {
                status: 402,
                headers: {
                    "Content-Type": "application/json",
                    "WWW-Authenticate": `L402 target="${options.requiredGatewayAccount}", min_motes="${options.minimumPaymentMotes}"`
                }
            });
        } catch (error: any) {
            const message = error instanceof Error ? error.message : String(error);
            return new Response(JSON.stringify({ error: `Internal Server Error: ${message}` }), {
                status: 500,
                headers: { "Content-Type": "application/json" }
            });
        }
    };
}
