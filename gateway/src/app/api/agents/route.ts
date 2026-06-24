import { NextResponse } from "next/server";
import { AgentRegistry } from "@/lib/agent_registry";

export const dynamic = "force-dynamic";

export async function GET() {
	try {
		const agents = AgentRegistry.getAll();

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
