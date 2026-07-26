import { verifyTransactionOnChain } from "../services/casper.js";
/**
 * Express middleware for protecting routes with L402 payments.
 *
 * @param options - Configuration for verification (gateway account, minimum motes, etc.)
 * @returns An Express middleware function
 */
export function expressX402Middleware(options) {
    return async (req, res, next) => {
        const authHeader = req.headers.authorization;
        if (!authHeader || !authHeader.startsWith("L402 ")) {
            res.status(402)
                .set("WWW-Authenticate", `L402 target="${options.requiredGatewayAccount}", min_motes="${options.minimumPaymentMotes}"`)
                .json({ error: "L402 Payment Required" });
            return;
        }
        const txHash = authHeader.split(" ")[1];
        try {
            const verification = await verifyTransactionOnChain(txHash, options);
            if (verification.authorized) {
                // Attach the verified payer and payment ID to the request object for downstream routes
                req.x402 = {
                    payer: verification.payerPublicKey,
                    paymentId: verification.paymentId
                };
                next();
            }
            else {
                res.status(402)
                    .set("WWW-Authenticate", `L402 target="${options.requiredGatewayAccount}", min_motes="${options.minimumPaymentMotes}"`)
                    .json({ error: verification.reason });
            }
        }
        catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            res.status(500).json({ error: `Internal Server Error: ${message}` });
        }
    };
}
