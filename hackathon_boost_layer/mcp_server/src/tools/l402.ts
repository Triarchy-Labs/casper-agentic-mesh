import { z } from "zod";
import { sharedCasperClient as client } from "../casper-provider.js";

const l402Schema = z.object({
    txHash: z.string().describe("The Casper transaction hash to verify"),
    pollingInterval: z.number().optional().default(5000).describe("Polling interval in ms"),
    maxWaitTime: z.number().max(60000).optional().default(30000).describe("Maximum time to wait for confirmation in ms")
});

export const verifyL402PaymentTool = {
    name: "verify_l402_payment",
    description: "Verifies if an L402 payment transaction (CSPR transfer) has successfully completed. Returns the target account, amount, and paymentId (if any) to prevent spoofing.",
    schema: l402Schema,
    handler: async (args: z.infer<typeof l402Schema>) => {
        const startTime = Date.now();
        
        return new Promise((resolve, reject) => {
            const checkTx = async () => {
                try {
                    const result = await client.executeWithFailover(async (c) => c.getDeployInfo(args.txHash));
                    
                    if (result.execution_results && result.execution_results.length > 0) {
                        const execResult = result.execution_results[0].result;
                        if (execResult.Success) {
                            // WORMHOLE FIX: Do not just return success. Extract the target and amount!
                            let target = null;
                            let amount = null;
                            let paymentId = null;

                            if (result.deploy.session && result.deploy.session.Transfer) {
                                const tArgs = result.deploy.session.Transfer.args;
                                for (const arg of tArgs) {
                                    const argName = arg[0];
                                    const argValue = arg[1] as any;
                                    if (argName === 'target') target = argValue.parsed;
                                    if (argName === 'amount') amount = argValue.parsed;
                                    if (argName === 'id') paymentId = argValue.parsed;
                                }
                            }

                            resolve({ 
                                status: "success", 
                                execution: "success",
                                transfer: {
                                    target,
                                    amountMotes: amount,
                                    paymentId
                                },
                                warning: target === null ? "This transaction does not appear to be a standard native Transfer." : undefined
                            });
                        } else {
                            resolve({ status: "failed", execution: "failed", reason: execResult.Failure?.error_message });
                        }
                        return;
                    }
                    
                    if (Date.now() - startTime > args.maxWaitTime) {
                        resolve({ status: "timeout", message: "Transaction is in mempool but execution timed out" });
                        return;
                    }
                    
                    setTimeout(checkTx, args.pollingInterval);
                } catch (error: any) {
                    const isNotFound = error.message && error.message.includes("deploy not known");
                    if (isNotFound) {
                         if (Date.now() - startTime > args.maxWaitTime) {
                             resolve({ status: "not_found", message: "Transaction not found on the network (could have been dropped before mempool)" });
                             return;
                         }
                         setTimeout(checkTx, args.pollingInterval);
                    } else {
                        reject(error);
                    }
                }
            };
            
            checkTx();
        });
    }
};
