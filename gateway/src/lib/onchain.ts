/**
 * Live on-chain reads from the deployed Triarchy contracts (server-side).
 * No mock data — every value comes from a Casper testnet RPC query.
 */

const RPC = process.env.CASPER_RPC_URL || "https://node.testnet.casper.network/rpc";

// Deployed oracle contract dictionaries (testnet).
const READINGS_UREF =
	process.env.CASPER_ORACLE_READINGS_UREF ||
	"uref-0635112c4d2ae2dd60d333bbcc5c9ec1858361101435a1e2d7a2bd7fd2242105-007";
const REPUTATION_UREF =
	process.env.CASPER_ORACLE_REPUTATION_UREF ||
	"uref-87be256170aab9d412b2e3ee649943ab082b07b1fcf40816c7a017183e8b4567-007";
const AGENT_ACCOUNT =
	process.env.CASPER_AGENT_ACCOUNT ||
	"334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867";

async function rpc(method: string, params: unknown): Promise<any> {
	const r = await fetch(RPC, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
		// Always hit the live ledger.
		cache: "no-store",
	});
	if (!r.ok) throw new Error(`RPC HTTP ${r.status}`);
	return r.json();
}

async function stateRootHash(): Promise<string> {
	const d = await rpc("chain_get_state_root_hash", []);
	return d.result.state_root_hash;
}

async function dictItemBytes(
	srh: string,
	seedUref: string,
	key: string,
): Promise<string | null> {
	const d = await rpc("state_get_dictionary_item", {
		state_root_hash: srh,
		dictionary_identifier: {
			URef: { seed_uref: seedUref, dictionary_item_key: key },
		},
	});
	return d?.result?.stored_value?.CLValue?.bytes ?? null;
}

/** Decode a CES/CLValue String (u32 LE length prefix + utf8 bytes). */
function decodeClString(hex: string): string {
	const bytes: number[] = [];
	for (let i = 0; i < hex.length; i += 2) bytes.push(parseInt(hex.slice(i, i + 2), 16));
	const len = bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24);
	return Buffer.from(bytes.slice(4, 4 + len)).toString("utf8");
}

/** Decode a little-endian u64 (low 53 bits, safe for counters) from a CLValue hex. */
function decodeClU64(hex: string): number {
	let v = 0;
	for (let i = 0; i < 8; i++) {
		v += parseInt(hex.slice(i * 2, i * 2 + 2), 16) * Math.pow(256, i);
	}
	return v;
}

/** Extract the micro-USD price from a "value=..;by=..;seq=.." reading record. */
function microUsdFromReading(reading: string | null): number | null {
	if (!reading) return null;
	const m = reading.match(/value=(\d+)/);
	return m ? parseInt(m[1], 10) : null;
}

export interface OnChainSnapshot {
	asset: string;
	reading: string | null; // raw "value=..;by=..;seq=.." record
	reputation: number | null;
	priceUsd: number | null; // CSPR price in USD, from the on-chain oracle
	/** RWA-pegged quote: a $5 bounty expressed in CSPR at the live oracle rate. */
	peg: { usd: number; cspr: number } | null;
	source: "casper-testnet";
	fetchedAt: string;
}

export async function getOnChainSnapshot(
	asset = "CSPR-USD",
	pegUsd = 5,
): Promise<OnChainSnapshot> {
	const srh = await stateRootHash();
	const [readingHex, repHex] = await Promise.all([
		dictItemBytes(srh, READINGS_UREF, asset),
		dictItemBytes(srh, REPUTATION_UREF, AGENT_ACCOUNT),
	]);
	const reading = readingHex ? decodeClString(readingHex) : null;
	const microUsd = microUsdFromReading(reading);
	const priceUsd = microUsd ? microUsd / 1_000_000 : null;
	// RWA peg: convert a USD target into CSPR using the live on-chain price.
	const peg = priceUsd
		? { usd: pegUsd, cspr: Math.round(pegUsd / priceUsd) }
		: null;
	return {
		asset,
		reading,
		reputation: repHex ? decodeClU64(repHex) : null,
		priceUsd,
		peg,
		source: "casper-testnet",
		fetchedAt: new Date().toISOString(),
	};
}
