import { NextResponse } from "next/server";
import os from "os";

export const dynamic = "force-dynamic"; // Ensure fresh OS telemetry on every request

export async function GET() {
    try {
        const cpus = os.cpus();
        const load = os.loadavg(); // Returns an array [1, 5, 15] minute load averages
        const freeMem = os.freemem();
        const totalMem = os.totalmem();
        const memPercent = ((totalMem - freeMem) / totalMem) * 100;
        
        // We simulate 7 WASI Nodes, mapping them to the real underlying OS CPU threads
        const nodes = Array.from({ length: 7 }).map((_, i) => {
            const cpu = cpus[i % cpus.length];
            // Simulate jitter for latency based on load
            const baseLatency = 10 + (load[0] * 5);
            const latency = Math.max(2, Math.floor(baseLatency + (Math.random() * 12)));
            
            // If memory is critically high or this specific node fails a harsh random check, it breaches
            const isBreached = memPercent > 95 || (Math.random() < 0.05 && i === 6);

            return {
                id: i + 1,
                cluster: `CORE_${i}`,
                speed: cpu.speed,
                latency,
                status: isBreached ? "BREACHED" : (load[0] > 0.5 ? "COMPUTING" : "IDLE"),
            };
        });

        return NextResponse.json({
            status: "ok",
            system: {
                load: load[0].toFixed(2),
                mem: memPercent.toFixed(1),
                uptime: os.uptime(),
            },
            nodes
        });
    } catch (e) {
        return NextResponse.json({ error: "Failed to read OS telemetry" }, { status: 500 });
    }
}
