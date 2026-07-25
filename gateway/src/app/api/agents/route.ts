import { NextResponse } from "next/server";
import { AgentRegistry } from "@/lib/agent_registry";
import { getOnChainSnapshot } from "@/lib/onchain";

export const dynamic = "force-dynamic";

export async function GET() {
	try {
		const agents = AgentRegistry.getAll();

		// THE LIVE ONE (audit item C6): the real registered agent, reputation read from the
		// Casper testnet ledger on every call. If the chain is unreachable we simply omit it —
		// never a stale or invented number.
		try {
			const snap = (await Promise.race([
				getOnChainSnapshot(),
				new Promise((_, rej) => setTimeout(() => rej(new Error("timeout")), 6000)),
			])) as Awaited<ReturnType<typeof getOnChainSnapshot>>;
			if (snap && snap.reputation !== null) {
				agents.unshift({
					id: "sentinel_334f6577 · LIVE",
					capabilities: ["On-chain oracle feeds", "Escrow lifecycle (proven)"],
					status: "active",
					reputationScore: snap.reputation,
					tasksCompleted: snap.reputation, // reputation mints 1:1 from settled outcomes
					csprSettled: 10, // the documented escrow release (DEPLOYMENTS.md)
					lastActiveTracker: snap.fetchedAt,
					stakedCollateralCspr: 0,
					passportId: "on-chain · 334f6577…4867",
				});
			}
		} catch { /* ledger unreachable — list stays registry-only */ }

		// Sort by reputation (desc)
		const sortedAgents = agents.sort(
			(a, b) => b.reputationScore - a.reputationScore,
		);

		return NextResponse.json(
			{
				organization: "Triarchy Labs",
				protocol: "x402 Arbitrage Mesh",
				total_active: sortedAgents.filter((a) => a.status === "active").length,
				agents: sortedAgents,
			},
			{ status: 200 },
		);
	} catch (e: unknown) {
		const err = e as Error;
		return NextResponse.json(
			{ error: err.message || "Registry parsing failed" },
			{ status: 500 },
		);
	}
}
