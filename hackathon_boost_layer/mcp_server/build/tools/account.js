import { z } from "zod";
import { sharedCasperClient as client } from "../casper-provider.js";
const accountSchema = z.object({
    accountHash: z.string().describe("The account hash (with or without 'account-hash-' prefix)")
});
export const getAccountBalanceTool = {
    name: "get_account_balance",
    description: "Fetches the real-time CSPR balance of any Casper account using their Account Hash via resilient multi-node JSON-RPC queries.",
    schema: accountSchema,
    handler: async (args) => {
        try {
            let hash = args.accountHash.trim();
            if (hash.startsWith("account-hash-")) {
                hash = hash.substring("account-hash-".length);
            }
            const balance = await client.getAccountBalance(hash);
            return {
                accountHash: hash,
                balanceMotes: balance.toString(),
                balanceCSPR: (Number(balance) / 1_000_000_000).toString(),
            };
        }
        catch (error) {
            throw new Error(`Failed to get account balance: ${error.message}`);
        }
    }
};
