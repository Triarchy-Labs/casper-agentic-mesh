// Triarchy In-Memory Agent Registry
// Absorbed from trionlabs (Stellar 8004) architecture, but designed for real-time memory instead of on-chain CRUD.

export interface AgentRecord {
	id: string; // The specific agent UUID or client_id
	capabilities: string[]; // e.g., ["DeFi", "Arbitrage"]
	status: "active" | "quarantined" | "offline";
	reputationScore: number; // 0 to 100
	tasksCompleted: number;
	usdcSettled: number;
	lastActiveTracker: string; // ISO timestamp
}

class TriarchyAgentRegistry {
	private baseAgents: Map<string, AgentRecord> = new Map();

	constructor() {
		// Scaffold initial competitive agents
		this.register({
			id: "agent_alpha_arbitrage",
			capabilities: ["DEX Arbitrage", "Flash Loans"],
			status: "active",
			reputationScore: 99,
			tasksCompleted: 1042,
			usdcSettled: 420.5,
			lastActiveTracker: new Date().toISOString(),
		});

		this.register({
			id: "credio_risk_monitor",
			capabilities: ["DeFi Position Exit", "Risk Management"],
			status: "active",
			reputationScore: 95,
			tasksCompleted: 450,
			usdcSettled: 120.0,
			lastActiveTracker: new Date().toISOString(),
		});

		this.register({
			id: "malicious_node_x9",
			capabilities: ["Phishing", "Dusting"],
			status: "quarantined",
			reputationScore: 12,
			tasksCompleted: 5,
			usdcSettled: 0.0,
			lastActiveTracker: new Date().toISOString(),
		});
	}

	public register(agent: AgentRecord) {
		this.baseAgents.set(agent.id, agent);
	}

	public getAgent(id: string): AgentRecord | undefined {
		return this.baseAgents.get(id);
	}

	public getAll(): AgentRecord[] {
		return Array.from(this.baseAgents.values());
	}

	public updateStats(id: string, feeUsdc: number) {
		const agent = this.baseAgents.get(id);
		if (agent) {
			agent.tasksCompleted += 1;
			agent.usdcSettled += feeUsdc;
			agent.lastActiveTracker = new Date().toISOString();
			// Increase reputation slightly on successful task
			if (agent.reputationScore < 100) {
				agent.reputationScore = Math.min(100, agent.reputationScore + 0.1);
			}
		} else {
			// Auto-register new dynamically discovered agents hitting the gateway
			this.register({
				id,
				capabilities: ["Dynamic Routing"],
				status: "active",
				reputationScore: 80,
				tasksCompleted: 1,
				usdcSettled: feeUsdc,
				lastActiveTracker: new Date().toISOString(),
			});
		}
	}
}

// Global singleton instance
export const AgentRegistry = new TriarchyAgentRegistry();
