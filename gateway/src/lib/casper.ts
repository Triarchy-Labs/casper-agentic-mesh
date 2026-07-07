/**
 * Casper RPC Payment Validator (Testnet)
 *
 * Enforces real on-chain verification for L402 / x402 payments. There is NO
 * mock or bypass path. A payment is valid ONLY if its transaction:
 *   1. exists on the Casper ledger,
 *   2. executed successfully,
 *   3. contains a transfer TO the platform account, and
 *   4. that transfer is >= the required amount.
 *
 * (3) and (4) close the "any successful hash unlocks the API" gap: an unrelated
 * successful transaction, or one paying someone else, is rejected.
 */

const CASPER_TESTNET_RPC =
	process.env.CASPER_RPC_URL || "https://node.testnet.casper.network/rpc";

// Account-hash that must receive the payment. Defaults to the platform key's
// account-hash (public key 013d8de7…); override via env in other deployments.
const PLATFORM_ACCOUNT_HASH = (
	process.env.CASPER_PLATFORM_ACCOUNT_HASH ||
	"334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867"
).toLowerCase();

// Minimum motes a payment must transfer to the platform (default 2.5 CSPR — the
// native-transfer floor used by lib/pay.ts). 1 CSPR = 1e9 motes.
const MIN_PAYMENT_MOTES = BigInt(
	process.env.CASPER_MIN_PAYMENT_MOTES || "2500000000",
);

export interface PaymentValidationResult {
	valid: boolean;
	error?: string;
	amount?: number;
	currency?: string;
}

interface LedgerResult {
	found: boolean;
	success: boolean;
	pending: boolean;
	transfers: Transfer[];
}

interface Transfer {
	toAccountHash: string | null;
	amount: bigint;
}

async function rpcCall(method: string, params: unknown): Promise<any> {
	const response = await fetch(CASPER_TESTNET_RPC, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
	});
	if (!response.ok) throw new Error(`Casper RPC HTTP ${response.status}`);
	return response.json();
}

/** Normalizes a transfer `to`/`from` field to a bare lowercase account-hash hex. */
function normalizeAccountHash(v: unknown): string | null {
	let s: string | null = null;
	if (typeof v === "string") s = v;
	else if (v && typeof v === "object" && "AccountHash" in (v as any)) {
		s = String((v as any).AccountHash);
	}
	if (!s) return null;
	return s.replace(/^account-hash-/, "").toLowerCase();
}

/** Extracts the transfer records from a Casper 2.0 Version2 execution result. */
function extractTransfers(execResult: any): Transfer[] {
	const v2 = execResult?.Version2 ?? execResult;
	const raw = v2?.transfers;
	if (!Array.isArray(raw)) return [];
	return raw.map((t: any) => {
		const rec = t?.Version2 ?? t;
		let amount = BigInt(0);
		try {
			amount = BigInt(rec?.amount ?? 0);
		} catch {
			amount = BigInt(0);
		}
		return { toAccountHash: normalizeAccountHash(rec?.to), amount };
	});
}

/**
 * Reads a V1 transaction (or Deploy) execution status + transfers.
 * The hash may be wrapped as Version1 or Deploy depending on submission path.
 */
async function getLedgerResult(txHash: string): Promise<LedgerResult> {
	for (const wrapper of ["Version1", "Deploy"] as const) {
		try {
			const data = await rpcCall("info_get_transaction", {
				transaction_hash: { [wrapper]: txHash },
			});
			if (data.error) continue;
			const info = data.result?.execution_info;
			if (info === null || info === undefined) {
				return { found: true, success: false, pending: true, transfers: [] };
			}
			const execResult = info.execution_result;
			const res = execResult?.Version2 ?? execResult;
			const success = res && res.error_message == null;
			return {
				found: true,
				success: !!success,
				pending: false,
				transfers: extractTransfers(execResult),
			};
		} catch {
			// try next wrapper
		}
	}
	return { found: false, success: false, pending: false, transfers: [] };
}

/**
 * Validates a Casper payment by its transaction hash.
 * @param txHash         The transaction/deploy hash supplied via the L402 header.
 * @param requiredAmount Minimum CSPR required for the task tier.
 * @param _expectedMemo  Reserved for task/client correlation (transfer id).
 */
export async function validateCasperPayment(
	txHash: string,
	requiredAmount: number,
	_expectedMemo: string,
): Promise<PaymentValidationResult> {
	// A real Casper transaction hash is exactly 64 hex chars. No exceptions.
	if (!/^[0-9a-fA-F]{64}$/.test(txHash)) {
		return {
			valid: false,
			error: "Invalid payment hash: a 64-char Casper transaction hash is required.",
		};
	}

	try {
		const ledger = await getLedgerResult(txHash);

		if (!ledger.found) {
			return { valid: false, error: "Transaction not found on Casper ledger." };
		}
		if (ledger.pending) {
			return { valid: false, error: "Transaction is still pending execution." };
		}
		if (!ledger.success) {
			return { valid: false, error: "Transaction execution failed on Casper ledger." };
		}

		// Required amount in motes: at least the platform gate, and at least the
		// task tier price if that is higher.
		const requiredMotes =
			requiredAmount > 0
				? BigInt(Math.ceil(requiredAmount * 1e9))
				: BigInt(0);
		const gate =
			requiredMotes > MIN_PAYMENT_MOTES ? requiredMotes : MIN_PAYMENT_MOTES;

		// Find a transfer that pays the platform account at least the gate amount.
		const match = ledger.transfers.find(
			(t) => t.toAccountHash === PLATFORM_ACCOUNT_HASH && t.amount >= gate,
		);

		if (!match) {
			const toPlatform = ledger.transfers.some(
				(t) => t.toAccountHash === PLATFORM_ACCOUNT_HASH,
			);
			if (toPlatform) {
				return {
					valid: false,
					error: `Underpaid: transfer to the platform is below the ${gate} motes gate.`,
				};
			}
			return {
				valid: false,
				error: "No transfer to the platform account found in this transaction.",
			};
		}

		return {
			valid: true,
			amount: Number(match.amount) / 1e9,
			currency: "CSPR",
		};
	} catch (e: unknown) {
		const msg = e instanceof Error ? e.message : String(e);
		console.error("[CASPER VALIDATION ERROR]:", msg);
		return { valid: false, error: `Casper RPC validation error: ${msg}` };
	}
}
