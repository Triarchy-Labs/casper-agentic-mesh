/**
 * Real on-chain payment via Casper Wallet — no mocks.
 *
 * Builds a native CSPR transfer to the platform account, has the user sign it
 * with the Casper Wallet browser extension, submits it to the testnet node, and
 * returns the real deploy hash. That hash is then verified server-side against
 * the live ledger (see lib/casper.ts) before any task is executed.
 */
import { CasperServiceByJsonRPC, DeployUtil, CLPublicKey } from "casper-js-sdk";

const RPC =
	process.env.NEXT_PUBLIC_CASPER_RPC_URL ||
	"https://node.testnet.casper.network/rpc";
const CHAIN = process.env.NEXT_PUBLIC_CASPER_CHAIN || "casper-test";
// Platform account that receives the L402 micro-payment (public key hex).
const PLATFORM_PUBKEY =
	process.env.NEXT_PUBLIC_CASPER_PLATFORM_PUBKEY ||
	"013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35";

// Native transfers have a 2.5 CSPR minimum on Casper.
const PAYMENT_MOTES = "100000000"; // 0.1 CSPR gas
const TRANSFER_MOTES = "2500000000"; // 2.5 CSPR

/**
 * Returns the 64-char hex deploy hash of a successfully submitted payment.
 * Throws if the wallet is missing, the user cancels, or submission fails.
 */
export async function payForTask(
	_intent: string,
	senderPubHex: string,
): Promise<string> {
	if (typeof window === "undefined" || !window.casperWallet) {
		throw new Error("CASPER_WALLET_REQUIRED");
	}

	const sender = CLPublicKey.fromHex(senderPubHex);
	const recipient = CLPublicKey.fromHex(PLATFORM_PUBKEY);

	const transferId = Date.now();
	const session = DeployUtil.ExecutableDeployItem.newTransfer(
		TRANSFER_MOTES,
		recipient,
		undefined,
		transferId,
	);
	const payment = DeployUtil.standardPayment(PAYMENT_MOTES);
	const deploy = DeployUtil.makeDeploy(
		new DeployUtil.DeployParams(sender, CHAIN),
		session,
		payment,
	);

	const deployJson = DeployUtil.deployToJson(deploy);
	const signResult = await window.casperWallet.sign(
		JSON.stringify(deployJson),
		senderPubHex,
	);
	if (signResult?.cancelled) {
		throw new Error("USER_CANCELLED_PAYMENT");
	}

	const signedDeploy = DeployUtil.setSignature(
		deploy,
		signResult.signature,
		sender,
	);

	const client = new CasperServiceByJsonRPC(RPC);
	const { deploy_hash } = await client.deploy(signedDeploy);
	return deploy_hash;
}
