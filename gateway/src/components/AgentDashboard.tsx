"use client";
import { motion } from "framer-motion";
import { useEffect, useState } from "react";

interface Agent {
	id: string;
	capabilities: string[];
	status: string;
	reputationScore: number;
	tasksCompleted: number;
	usdcSettled: number;
}

export default function AgentDashboard() {
	const [data, setData] = useState<{
		total_active: number;
		agents: Agent[];
	} | null>(null);

	useEffect(() => {
		const fetchAgents = async () => {
			try {
				const res = await fetch("/api/agents");
				if (res.ok) {
					setData(await res.json());
				}
			} catch (e) {
				console.error(e);
			}
		};
		fetchAgents();
		const interval = setInterval(fetchAgents, 5000);
		return () => clearInterval(interval);
	}, []);

	return (
		<motion.div
			initial={{ opacity: 0, y: 50 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ delay: 2.2, duration: 1 }}
			style={{
				position: "relative",
				zIndex: 10,
				padding: "2rem",
				width: "100%",
				maxWidth: "1000px",
				margin: "0 auto 5rem",
			}}
		>
			<h2
				style={{
					color: "rgba(255,255,255,0.85)",
					fontFamily: "'Helvetica Now Display', 'Inter', sans-serif",
					fontSize: "1.5rem",
					letterSpacing: "0.2em",
					borderBottom: "1px solid rgba(255,255,255,0.3)",
					paddingBottom: "1rem",
				}}
			>
				:: LIVE AGENT REGISTRY ::
			</h2>

			<div
				style={{
					display: "grid",
					gridTemplateColumns: "repeat(auto-fit, minmax(300px, 1fr))",
					gap: "2rem",
					marginTop: "2rem",
				}}
			>
				{data?.agents.map((agent) => (
					<div
						key={agent.id}
						style={{
							background: "rgba(0, 15, 0, 0.4)",
							border: `1px solid ${agent.status === "active" ? "rgba(255, 255, 255, 0.4)" : agent.status === "quarantined" ? "rgba(255, 0, 60, 0.4)" : "rgba(100, 100, 100, 0.4)"}`,
							borderRadius: "4px",
							padding: "1.5rem",
							backdropFilter: "blur(10px)",
							fontFamily: "'Helvetica Now Display', 'Inter', sans-serif",
						}}
					>
						<div
							style={{
								display: "flex",
								justifyContent: "space-between",
								alignItems: "center",
								marginBottom: "1rem",
							}}
						>
							<span
								style={{
									color: "#fff",
									fontSize: "1.2rem",
									fontWeight: "bold",
								}}
							>
								{agent.id}
							</span>
							<span
								style={{
									color:
										agent.status === "active"
											? "rgba(255,255,255,0.85)"
											: agent.status === "quarantined"
												? "#ff003c"
												: "#888",
									textTransform: "uppercase",
									fontSize: "0.8rem",
									padding: "0.2rem 0.5rem",
									border: `1px solid ${agent.status === "active" ? "rgba(255,255,255,0.85)" : agent.status === "quarantined" ? "#ff003c" : "#888"}`,
								}}
							>
								{agent.status}
							</span>
						</div>
						<div
							style={{
								color: "#aaa",
								fontSize: "0.9rem",
								marginBottom: "1rem",
							}}
						>
							[{agent.capabilities.join(", ")}]
						</div>
						<div
							style={{
								display: "flex",
								justifyContent: "space-between",
								color: "rgba(255,255,255,0.85)",
								fontSize: "0.9rem",
							}}
						>
							<span>REP: {agent.reputationScore.toFixed(1)}</span>
							<span>TASKS: {agent.tasksCompleted}</span>
							<span>USDC: ${agent.usdcSettled.toFixed(2)}</span>
						</div>
					</div>
				))}
				{!data && (
					<div style={{ color: "#888", fontFamily: "'Helvetica Now Display', 'Inter', sans-serif" }}>
						Scanning registry...
					</div>
				)}
			</div>
		</motion.div>
	);
}
