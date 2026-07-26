export class CSPRCloudRestClient {
    baseUrl;
    apiKey;
    constructor(config) {
        this.apiKey = config.apiKey;
        const network = config.network || "testnet";
        this.baseUrl = `https://api.${network}.cspr.cloud`;
    }
    async request(endpoint, retries = 3) {
        const url = `${this.baseUrl}${endpoint}`;
        let lastError;
        for (let i = 0; i < retries; i++) {
            try {
                // Native Web Fetch! No node-fetch dependency. 100% universal.
                const response = await fetch(url, {
                    headers: {
                        "authorization": this.apiKey,
                        "accept": "application/json"
                    }
                });
                if (response.status === 429 || response.status >= 500) {
                    await response.text();
                    throw new Error(`Transient API Error: ${response.status}`);
                }
                if (!response.ok) {
                    await response.text();
                    throw new Error(`CSPR.cloud API Error: ${response.status} ${response.statusText}`);
                }
                return await response.json();
            }
            catch (error) {
                lastError = error;
                const isTransient = error instanceof Error && (error.message.includes("Transient") ||
                    error.message.includes("ECONNRESET") ||
                    error.message.includes("ETIMEDOUT") ||
                    error.message.includes("ENOTFOUND") ||
                    error.message.includes("EAI_AGAIN") ||
                    error.message.includes("fetch failed") // Native fetch error format
                );
                if (!isTransient) {
                    throw error;
                }
                if (i < retries - 1) {
                    await new Promise(r => setTimeout(r, Math.pow(2, i) * 1000));
                }
            }
        }
        throw lastError;
    }
    async getEvents(contractPackageHash, limit = 10, cursor) {
        let endpoint = `/contract-packages/${contractPackageHash}/events?limit=${limit}`;
        if (cursor) {
            endpoint += `&page=${cursor}`;
        }
        return this.request(endpoint);
    }
}
