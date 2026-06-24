import { NextResponse } from "next/server";
import { promises as fs } from "node:fs";
import path from "node:path";

const DB_FILE = path.join(process.cwd(), "bounties.json");

interface Bounty {
    id: string;
    title: string;
    bounty: string;
    status: "OPEN" | "IN PROGRESS" | "LOCKED" | "COMPLETED";
    issuer: string;
    skills: string[];
    difficulty: string;
}

// STUB: Fallback local file-based database. In production, this requires atomic file locking or a real database (e.g., PostgreSQL) to prevent write race conditions during concurrent operations.
// Ensure the db file exists, initialized with default values from the UI
async function getBounties(): Promise<Bounty[]> {
    try {
        const data = await fs.readFile(DB_FILE, "utf-8");
        return JSON.parse(data);
    } catch {
        const defaults: Bounty[] = [
            { id: "Q-1049", title: "Delta Neutral Arbitrage Audit", bounty: "5,000 USDC", status: "OPEN", issuer: "Triarchy-Labs", skills: ["Rust", "Casper", "DeFi"], difficulty: "GOD-TIER" },
            { id: "Q-1021", title: "WASM Payload Refactoring", bounty: "850 USDC", status: "IN PROGRESS", issuer: "Anonymous", skills: ["WebAssembly", "C++"], difficulty: "A-TIER" },
            { id: "Q-0992", title: "Frontend Telemetry Injection", bounty: "200 USDC", status: "OPEN", issuer: "Casper Network", skills: ["Next.js", "React"], difficulty: "B-TIER" },
        ];
        await fs.writeFile(DB_FILE, JSON.stringify(defaults, null, 2));
        return defaults;
    }
}

async function saveBounties(bounties: Bounty[]) {
    await fs.writeFile(DB_FILE, JSON.stringify(bounties, null, 2));
}

export async function GET() {
    const bounties = await getBounties();
    return NextResponse.json(bounties);
}

export async function POST(req: Request) {
    try {
        const body = await req.json();
        const bounties = await getBounties();
        
        const newBounty: Bounty = {
            id: `Q-${Math.floor(Math.random() * 9000) + 1000}`,
            title: body.title || "Unknown Task",
            bounty: body.usdc ? `${body.usdc} USDC` : "0 USDC",
            status: "OPEN",
            issuer: body.issuer || "Anonymous",
            skills: body.skills || ["General"],
            difficulty: body.difficulty || "C-TIER"
        };
        
        bounties.unshift(newBounty); // Add to top
        await saveBounties(bounties);
        
        return NextResponse.json(newBounty, { status: 201 });
    } catch (e: unknown) {
        return NextResponse.json({ error: (e instanceof Error ? e.message : String(e)) }, { status: 500 });
    }
}

export async function PATCH(req: Request) {
    try {
        const { id, status } = await req.json();
        const bounties = await getBounties();
        
        const bountyIndex = bounties.findIndex(b => b.id === id);
        if (bountyIndex === -1) return NextResponse.json({ error: "Bounty not found" }, { status: 404 });
        
        bounties[bountyIndex].status = status;
        await saveBounties(bounties);
        
        return NextResponse.json(bounties[bountyIndex]);
    } catch (e: unknown) {
        return NextResponse.json({ error: (e instanceof Error ? e.message : String(e)) }, { status: 500 });
    }
}
