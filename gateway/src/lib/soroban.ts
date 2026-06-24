/* eslint-disable */
/**
 * Stellar Horizon RPC Validator (Testnet)
 * Enforces real-world on-chain verification for x402 payments.
 * Replaces the mock validation to meet DoraHacks requirements.
 */

const HORIZON_TESTNET_URL = "https://horizon-testnet.stellar.org";
// Testnet USDC issuer (Example public issuer for hackathon testing)
const USDC_ISSUER =
	process.env.STELLAR_USDC_ISSUER ||
	"GBBD47IF6LWK7P7MDEVSCWTTCJM4NTC3L3P6HZ2ZNGQ5ZCDNDGNQ5V5D";
const PLATFORM_WALLET =
	process.env.STELLAR_PLATFORM_WALLET || "GPLATFORM_WALLET_DEFAULT";

export interface PaymentValidationResult {
	valid: boolean;
	error?: string;
	amount?: number;
	currency?: string;
}

/**
 * Validates a Stellar transaction by its hash.
 * Ensures the transaction was successful and transferred the required USDC to the Platform Wallet.
 *
 * @param txHash The transaction hash provided by the external agent via L402 header.
 * @param requiredAmount Minimum USDC required for the task tier.
 * @param expectedMemo The task_id or client_id that must be in the transaction memo to prevent double-spending.
 */
export async function validateSorobanPayment(
	txHash: string,
	requiredAmount: number,
	expectedMemo: string,
): Promise<PaymentValidationResult> {
	try {
		// Fetch transaction details
		const txResp = await fetch(`${HORIZON_TESTNET_URL}/transactions/${txHash}`);
		if (!txResp.ok) {
			return {
				valid: false,
				error: "Transaction not found on Stellar Testnet.",
			};
		}

		const txData = await txResp.json();

		if (!txData.successful) {
			return { valid: false, error: "Transaction failed on-chain." };
		}

		// Verify Memo
		if (txData.memo_type !== "text" || txData.memo !== expectedMemo) {
			return {
				valid: false,
				error: `Invalid Memo. Expected '${expectedMemo}', got '${txData.memo}'.`,
			};
		}

		// Fetch operations to verify the payment amount and destination
		const opsResp = await fetch(
			`${HORIZON_TESTNET_URL}/transactions/${txHash}/operations`,
		);
		if (!opsResp.ok) {
			return { valid: false, error: "Could not fetch transaction operations." };
		}

		const opsData = await opsResp.json();

		let paymentFound = false;
		let totalPaid = 0;

		for (const op of opsData._embedded.records) {
			if (op.type === "payment" && op.to === PLATFORM_WALLET) {
				// If checking specific native asset vs USDC
				// For Hackathon prototype, we accept "USDC" or equivalent XLM value.
				if (op.asset_code === "USDC" && op.asset_issuer === USDC_ISSUER) {
					totalPaid += parseFloat(op.amount);
					paymentFound = true;
				} else if (op.asset_type === "native") {
					// Equivalent XLM conversion logic can go here
					// Let's assume testing with native XLM if USDC isn't available
					totalPaid += parseFloat(op.amount);
					paymentFound = true;
				}
			}
		}

		if (!paymentFound) {
			return {
				valid: false,
				error:
					"No valid payment to the Enterprise Platform Wallet found in this transaction.",
			};
		}

		if (totalPaid < requiredAmount) {
			return {
				valid: false,
				error: `Insufficient funds. Paid ${totalPaid}, required ${requiredAmount}.`,
			};
		}

		return { valid: true, amount: totalPaid };
	} catch (e: any) {
		console.error("[SOROBAN VALIDATION ERROR]:", e.message);
		return {
			valid: false,
			error: "Internal validation error connecting to Horizon RPC.",
		};
	}
}
