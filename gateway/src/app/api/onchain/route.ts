import { NextResponse } from "next/server";
import { getOnChainSnapshot } from "@/lib/onchain";

// Live on-chain snapshot (oracle reading + agent reputation) from Casper testnet.
export const dynamic = "force-dynamic";

export async function GET(req: Request) {
	const asset = new URL(req.url).searchParams.get("asset") || "CSPR-USD";
	try {
		const snapshot = await getOnChainSnapshot(asset);
		return NextResponse.json(snapshot);
	} catch (e: unknown) {
		const msg = e instanceof Error ? e.message : String(e);
		return NextResponse.json({ error: msg, source: "casper-testnet" }, { status: 502 });
	}
}
