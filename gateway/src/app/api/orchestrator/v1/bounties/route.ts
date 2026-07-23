import { NextRequest, NextResponse } from "next/server";
import { POST as hireHandler } from "@/app/api/hire/route";

/**
 * Autonomous Bounty Ingestion — Bot A2A Hook
 * Endpoint: POST /api/orchestrator/v1/bounties
 * Compatible with curl / ElizaOS / OpenClaw agents
 *
 * Bridges the A2A protocol from the Bounty Board UI into /api/hire by invoking
 * the hire handler DIRECTLY (in-process). No network fetch is involved, so no
 * request-derived URL exists — the SSRF class (js/request-forgery: attacker
 * steering the origin via the Host header) is eliminated by construction.
 */
export async function POST(req: NextRequest) {
    try {
        const body = await req.json();
        const { bot_pubkey, action, quest_id, description, bounty_cspr, client_id } = body;

        // Validate required fields
        if (!action) {
            return NextResponse.json(
                { error: "Missing required field: action" },
                { status: 400 }
            );
        }

        // Only support 'claim' action for now
        if (action !== "claim" && action !== "submit") {
            return NextResponse.json(
                { error: `Unknown action: ${action}. Supported: 'claim', 'submit'` },
                { status: 400 }
            );
        }

        // Build the x402 payload for /api/hire
        const hirePayload = {
            task_id: quest_id || `auto_${Date.now()}`,
            description: description || `Bot claiming quest: ${quest_id}`,
            bounty_cspr: bounty_cspr || 1.0,
            client_id: bot_pubkey || client_id || "autonomous_agent",
            payload_id: quest_id,
        };

        const txHash = req.headers.get("x-l402-txhash");
        if (!txHash) {
            return NextResponse.json(
                { error: "Payment Required. Please provide x-l402-txhash header." },
                { status: 402 }
            );
        }

        // Forward to /api/hire by calling the route handler in-process. The URL below is a
        // constant placeholder (the hire handler never reads req.url) — nothing about the
        // destination is derived from the incoming request.
        const hireResp = await hireHandler(
            new Request("http://internal/api/hire", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "x-l402-txhash": txHash,
                },
                body: JSON.stringify(hirePayload),
            })
        );

        const hireData = await hireResp.json();

        return NextResponse.json({
            bounty_id: hirePayload.task_id,
            action,
            agent: bot_pubkey || "anonymous_bot",
            result: hireData,
            protocol: "x402/L402",
            timestamp: new Date().toISOString(),
        }, { status: hireResp.ok ? 200 : hireResp.status });

    } catch (error: unknown) {
        return NextResponse.json(
            { error: "Orchestrator ingestion failed", details: error instanceof Error ? error.message : "Unknown error" },
            { status: 500 }
        );
    }
}

export async function GET() {
    return NextResponse.json({
        endpoint: "POST /api/orchestrator/v1/bounties",
        protocol: "x402/L402",
        description: "Autonomous bounty ingestion for AI agents (ElizaOS / OpenClaw / curl)",
        example: {
            bot_pubkey: "GXYZ...",
            action: "claim",
            quest_id: "Q-1049",
            bounty_cspr: 5.0,
        },
    });
}
