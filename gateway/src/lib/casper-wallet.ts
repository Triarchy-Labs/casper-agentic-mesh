// Casper Wallet bridge. The modern extension (v2.x) injects `window.CasperWalletProvider()` —
// a provider FACTORY — not the legacy `window.casperWallet` global. This broke the connect
// button on newer installs. We resolve whichever is present, so both old and new wallets work.

export interface CasperProvider {
	requestConnection: () => Promise<boolean | void>;
	getActivePublicKey: () => Promise<string>;
	isConnected: () => Promise<boolean>;
	disconnectFromSite?: () => Promise<boolean> | void;
	sign: (deployJson: string, publicKey: string) => Promise<{ cancelled?: boolean; signature?: Uint8Array; signatureHex?: string }>;
	signMessage?: (message: string, publicKey: string) => Promise<{ cancelled?: boolean; signature?: Uint8Array; signatureHex?: string } | string>;
}

export function getCasperProvider(): CasperProvider | null {
	if (typeof window === "undefined") return null;
	const w = window as unknown as {
		CasperWalletProvider?: (opts?: unknown) => CasperProvider;
		casperWallet?: CasperProvider;
	};
	// modern extension: a constructor that returns a fresh provider
	if (typeof w.CasperWalletProvider === "function") {
		try {
			return w.CasperWalletProvider();
		} catch {
			/* fall through to legacy */
		}
	}
	// legacy global
	if (w.casperWallet) return w.casperWallet;
	return null;
}

export function isCasperWalletInstalled(): boolean {
	return getCasperProvider() !== null;
}
