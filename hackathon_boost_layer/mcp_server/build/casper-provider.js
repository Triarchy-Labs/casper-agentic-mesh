import { CasperServiceByJsonRPC } from "casper-js-sdk";
export class ResilientCasperClient {
    rpcUrls;
    currentIndex = 0;
    clients;
    constructor() {
        if (process.env.CASPER_RPC_URLS) {
            this.rpcUrls = process.env.CASPER_RPC_URLS.split(',').map(u => u.trim());
        }
        else {
            this.rpcUrls = [
                "https://rpc.testnet.casperlabs.io/rpc",
                "https://testnet.casper-node.com/rpc",
                "https://casper-testnet.publicnode.com/rpc"
            ];
        }
        // Initialize and cache clients to keep connections alive and prevent memory allocation overhead
        this.clients = this.rpcUrls.map(url => new CasperServiceByJsonRPC(url));
    }
    getClient() {
        return this.clients[this.currentIndex];
    }
    rotateNode(failedIndex) {
        if (this.currentIndex === failedIndex) {
            this.currentIndex = (this.currentIndex + 1) % this.clients.length;
            console.error(`Rotated RPC node to: ${this.rpcUrls[this.currentIndex]}`);
        }
    }
    async executeWithFailover(operation) {
        const maxRetries = this.clients.length;
        let attempts = 0;
        let lastError;
        while (attempts < maxRetries) {
            const currentIdx = this.currentIndex;
            try {
                const client = this.getClient();
                // TCP Blackhole protection (10s timeout)
                const timeoutPromise = new Promise((_, reject) => {
                    setTimeout(() => reject(new Error("RPC Timeout (TCP Blackhole)")), 10000);
                });
                return await Promise.race([
                    operation(client),
                    timeoutPromise
                ]);
            }
            catch (error) {
                lastError = error;
                attempts++;
                if (attempts < maxRetries) {
                    this.rotateNode(currentIdx);
                    // Exponential backoff
                    await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempts) * 500));
                }
            }
        }
        const errorMessage = lastError instanceof Error ? lastError.message : String(lastError);
        throw new Error(`Failed after ${maxRetries} attempts. Last error: ${errorMessage}`);
    }
    async getAccountBalance(accountHash) {
        return this.executeWithFailover(async (client) => {
            const stateRootHash = await client.getStateRootHash();
            const blockState = await client.getBlockState(stateRootHash, `account-hash-${accountHash}`, []);
            if (!blockState.Account || !blockState.Account.mainPurse) {
                throw new Error("Could not find main purse for account");
            }
            const mainPurse = blockState.Account.mainPurse;
            const balanceMotes = await client.getAccountBalance(stateRootHash, mainPurse);
            return BigInt(balanceMotes.toString());
        });
    }
}
export const sharedCasperClient = new ResilientCasperClient();
