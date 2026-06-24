/* eslint-disable */

import * as fs from "node:fs";
import { NextResponse } from "next/server";
import { AgentRegistry } from "@/lib/agent_registry";
import { XRPLTransactor, TransactorContext } from "@/lib/xrpl-transactor";
import { replayGuard } from "@/lib/replay-guard";
import { spendingPolicy } from "@/lib/spending-policy";
import { isAllowedUrl } from "@/lib/security";

// Configuration: Environment-driven routing parameters
const LOCAL_EXECUTION_HOOK = process.env.LOCAL_EXECUTION_HOOK;
const ARBITRAGE_FEE_PCT = parseFloat(process.env.DYNAMIC_ROUTING_FEE || "0.00");
const ENTERPRISE_THRESHOLD = parseFloat(
	process.env.ENTERPRISE_THRESHOLD || "5.00",
);

export async function POST(req: Request) {
	try {
		// 1. Validate x402 (Payment Required) & L402 Subscription Hooks
		const authHeader = req.headers.get("authorization") || req.headers.get("x-l402-txhash");
		let txHash = req.headers.get("x-l402-txhash") || "";
		const l402Mode = req.headers.get("x-l402-mode");

		if (!txHash && authHeader && authHeader.startsWith("L402 ")) {
			txHash = authHeader.split(" ")[1];
		}

		if (!txHash) {
			if (l402Mode === "subscription") {
				return NextResponse.json(
					{ error: "Subscription Payment Required. Please provide valid recurring txhash via L402." },
					{ status: 402, headers: { "WWW-Authenticate": 'L402 invoice="soroban_subscription_required"' } }
				);
			}
		}

		let body: any = {};
		try { body = await req.json(); } catch(e) {}

		const ctx: TransactorContext = {
			txHash,
			bountyUsdc: body.bounty_usdc !== undefined ? parseFloat(body.bounty_usdc) : -1,
			description: body.description || "",
			clientId: body.client_id || "",
			taskId: body.task_id || ""
		};

		// --- PIPELINE STAGE 1: PREFLIGHT ---
		const preflight = XRPLTransactor.preflight(ctx);
		if (!preflight.valid) {
			return NextResponse.json({ error: preflight.error }, { 
                status: 402,
                headers: { "WWW-Authenticate": 'L402 invoice="soroban_payment_required"' }
            });
		}

		// --- PIPELINE STAGE 1.5: REPLAY GUARD (Assimilated from Toll) ---
		if (replayGuard.check(ctx.txHash)) {
			return NextResponse.json(
				{ error: "Payment signature already used.", detail: "Each txHash can only be used once. Signatures expire after 5 minutes." },
				{ status: 402 }
			);
		}

		// --- PIPELINE STAGE 1.6: SPENDING POLICY (Assimilated from Toll) ---
		const policyViolation = spendingPolicy.check(ctx.clientId || "anonymous", ctx.bountyUsdc);
		if (policyViolation) {
			return NextResponse.json(
				{ error: "Spending policy violation", reason: policyViolation, price: ctx.bountyUsdc },
				{ status: 429 }
			);
		}

		// --- PIPELINE STAGE 2: PRECLAIM ---
		const preclaim = await XRPLTransactor.preclaim(ctx);
		if (!preclaim.valid) {
			return NextResponse.json(
				{ 
					status: "rejected", 
					executor: "Security Node",
					message: preclaim.error,
					details: preclaim.details,
					usdc_charged: 0,
					usdc_refunded: preclaim.refundedUsdc || 0
				}, 
				{ status: 403 }
			);
		}

		// Mark txHash as used AFTER successful preclaim
		replayGuard.mark(ctx.txHash);

		const price = ctx.bountyUsdc;
		const description = ctx.description;
		const client_id = ctx.clientId;

		// --- PIPELINE STAGE 3: QUEUE CHECK ---
		const isQueueFull = XRPLTransactor.checkQueueState(LOCAL_EXECUTION_HOOK);

		// 2. TIER 1: MICRO-BOUNTY (Cloud LLM Sync via OpenRouter / Local Ollama fallback)
		if (price > 0 && price < ENTERPRISE_THRESHOLD && !isQueueFull) {
			try {
				const OPENROUTER_KEY = process.env.OPENROUTER_API_KEY;
				const OLLAMA_URL = process.env.OLLAMA_URL; // Only set for Mark 53 desktop

				let result: string | null = null;

				if (OPENROUTER_KEY) {
					// Production path: OpenRouter cloud API
					const orResp = await fetch("https://openrouter.ai/api/v1/chat/completions", {
						method: "POST",
						headers: {
							"Content-Type": "application/json",
							"Authorization": `Bearer ${OPENROUTER_KEY}`,
							"X-Title": "x402 Triarchy Gateway",
						},
						body: JSON.stringify({
							model: "anthropic/claude-3.5-sonnet",
							messages: [{ role: "user", content: `You are an AI agent fulfilling a micro-bounty via x402. Task: ${description}` }],
							max_tokens: 4096,
						}),
					});
					if (orResp.ok) {
						const orData = await orResp.json();
						result = orData.choices?.[0]?.message?.content || "No response";
					}
				} else if (OLLAMA_URL) {
					// Mark 53 desktop fallback: local Ollama
					const ollamaResp = await fetch(`${OLLAMA_URL}/api/generate`, {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify({
							model: "qwen3.5-9b-claude-os-instruct:latest",
							prompt: `You are an AI agent fulfilling a micro-bounty via x402. Task: ${description}`,
							stream: false,
						}),
					});
					if (ollamaResp.ok) {
						const ollamaData = await ollamaResp.json();
						result = ollamaData.response;
					}
				}

				if (result) {
					spendingPolicy.record(client_id || "micro_node_auto", price);
					AgentRegistry.updateStats(client_id || "micro_node_auto", price);
					return NextResponse.json(
						{
							status: "completed",
							executor: OPENROUTER_KEY ? "OpenRouter Cloud" : "Local Micro-Node",
							message: `Task completed synchronously via ${OPENROUTER_KEY ? "OpenRouter" : "Local LLM"}.`,
							usdc_charged: price,
							result,
						},
						{ status: 200 },
					);
				}
			} catch (_e) {
				console.warn(
					"LLM endpoint unavailable, escalating to P2P network.",
				);
			}
		}

		// 3. TIER 2: ENTERPRISE ROUTER (Priority local execution for high-value tasks)
		if (price >= ENTERPRISE_THRESHOLD && LOCAL_EXECUTION_HOOK && !isQueueFull) {
			// Routes to dedicated compute node when LOCAL_EXECUTION_HOOK is configured
			const secretPayload = {
				agent: "ENTERPRISE_NODE",
				command: "x402_intercept",
				bounty_usdc: price,
				origin: client_id || "openclaw_anon",
				instruction: description,
				timestamp: new Date().toISOString(),
			};

			try {
				// Atomic append for JSON payload < PIPE_BUF (uses O_APPEND internally via fs.promises)
				// With explicit file handle and lock behavior if needed, standard push is POSIX atomic.
				const fd = fs.openSync(
					LOCAL_EXECUTION_HOOK,
					fs.constants.O_WRONLY | fs.constants.O_CREAT | fs.constants.O_APPEND,
				);
				fs.writeSync(fd, `${JSON.stringify(secretPayload)}\n`);
				fs.closeSync(fd);

				spendingPolicy.record(client_id || "enterprise_node_auto", price);
				AgentRegistry.updateStats(client_id || "enterprise_node_auto", price);

				return NextResponse.json(
					{
						status: "accepted",
						executor: "Enterprise Sovereign Node",
						message: "Task accepted for priority local execution.",
						usdc_charged: price,
						fee: 0, // Zero routing fee for enterprise-tier tasks
					},
					{ status: 202 },
				);
			} catch (_e) {
				console.error("Local hook unavailable, falling back to P2P network.");
			}
		}

		// 4. TIER 3: P2P DELEGATION (Dynamic Load Balancing)
		// Routing fee is configurable via DYNAMIC_ROUTING_FEE env var
		// Under heavy load, dynamic fee adjusts to incentivize network capacity
		const activeFeePct = isQueueFull ? 0.5 : ARBITRAGE_FEE_PCT;
		const networkFee = price * activeFeePct;
		const foreignPrice = price - networkFee;

		// External bot call (Mock for video demo)
		let externalResult = "Awaiting response...";
		try {
			const resp = await fetch("http://127.0.0.1:3001/api/hire", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ task: description, amount: foreignPrice }),
			});
			const data = await resp.json();
			externalResult = data.result;
		} catch (_e) {
			console.warn(
				"Dummy bot at 3001 is offline. Run 'node dummy_external_bot.js' in a separate terminal.",
			);
		}

		AgentRegistry.updateStats(client_id || "remote_mercenary", foreignPrice);
		AgentRegistry.updateStats("gateway_router", networkFee); // The router itself keeps the fee

		return NextResponse.json(
			{
				status: "delegated",
				executor: "Peer-to-Peer Network Mercenary",
				message: isQueueFull
					? "Network busy, routed via \u00dcber Arbitrage protocol."
					: "Task delegated to idle network nodes.",
				usdc_charged: price,
				mercenary_paid: foreignPrice,
				network_fee: networkFee,
				external_agent_result: externalResult,
			},
			{ status: 200 },
		);
	} catch (error: any) {
		return NextResponse.json(
			{ error: "Internal Server Error", details: error.message },
			{ status: 500 },
		);
	}
}
