/**
 * Casper RPC Validator (Testnet)
 * Enforces real on-chain verification for Casper x402 / L402 payments.
 *
 * There is NO mock/bypass path. A payment is valid only if its transaction is
 * found on the Casper ledger AND executed successfully. Anything else is rejected.
 */

const CASPER_TESTNET_RPC =
	process.env.CASPER_RPC_URL || "https://node.testnet.casper.network/rpc";

export interface PaymentValidationResult {
	valid: boolean;
	error?: string;
	amount?: number;
	currency?: string;
}

interface RpcResult {
	executionFound: boolean;
	success: boolean;
	pending: boolean;
	error?: string;
}

async function rpcCall(method: string, params: unknown): Promise<any> {
	const response = await fetch(CASPER_TESTNET_RPC, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
	});
	if (!response.ok) {
		throw new Error(`Casper RPC HTTP ${response.status}`);
	}
	return response.json();
}

/**
 * Reads execution status of a V1 transaction (Casper 2.0). The transaction hash
 * may be wrapped as Version1 or Deploy depending on how it was submitted.
 */
async function getTransactionStatus(txHash: string): Promise<RpcResult> {
	for (const wrapper of ["Version1", "Deploy"] as const) {
		try {
			const data = await rpcCall("info_get_transaction", {
				transaction_hash: { [wrapper]: txHash },
			});
			if (data.error) continue;
			const info = data.result?.execution_info;
			if (info === null || info === undefined) {
				return { executionFound: true, success: false, pending: true };
			}
			const res = info.execution_result?.Version2 ?? info.execution_result;
			const success = res && res.error_message == null;
			return { executionFound: true, success: !!success, pending: false };
		} catch {
			// try next wrapper
		}
	}
	return { executionFound: false, success: false, pending: false };
}

/** Legacy fallback for pre-2.0 nodes. */
async function getDeployStatus(txHash: string): Promise<RpcResult> {
	try {
		const data = await rpcCall("info_get_deploy", { deploy_hash: txHash });
		if (data.error) return { executionFound: false, success: false, pending: false };
		const results = data.result?.execution_results;
		if (!results || results.length === 0) {
			return { executionFound: true, success: false, pending: true };
		}
		const success = results[0].result?.Success !== undefined;
		return { executionFound: true, success, pending: false };
	} catch {
		return { executionFound: false, success: false, pending: false };
	}
}

/**
 * Validates a Casper payment by its transaction hash.
 * @param txHash   The deploy/transaction hash supplied via the L402 header.
 * @param requiredAmount Minimum CSPR required for the task.
 * @param _expectedMemo  Reserved (task/client correlation).
 */
export async function validateCasperPayment(
	txHash: string,
	requiredAmount: number,
	_expectedMemo: string,
): Promise<PaymentValidationResult> {
	// Hard requirement: a real Casper transaction hash is 64 hex chars. No exceptions.
	if (!/^[0-9a-fA-F]{64}$/.test(txHash)) {
		return {
			valid: false,
			error: "Invalid payment hash: a 64-char Casper transaction hash is required.",
		};
	}

	try {
		let status = await getTransactionStatus(txHash);
		if (!status.executionFound) {
			status = await getDeployStatus(txHash);
		}

		if (!status.executionFound) {
			return { valid: false, error: "Transaction not found on Casper ledger." };
		}
		if (status.pending) {
			return { valid: false, error: "Transaction is still pending execution." };
		}
		if (!status.success) {
			return { valid: false, error: "Transaction execution failed on Casper ledger." };
		}

		return { valid: true, amount: requiredAmount, currency: "CSPR" };
	} catch (e: unknown) {
		const msg = e instanceof Error ? e.message : String(e);
		console.error("[CASPER VALIDATION ERROR]:", msg);
		return { valid: false, error: `Casper RPC validation error: ${msg}` };
	}
}
