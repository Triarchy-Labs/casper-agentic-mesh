import React from "react";
import { CarbonFabric } from "@/components/CarbonFabric";

export default function AppLoading() {
    return (
        <div className="min-h-screen flex flex-col items-center justify-center font-mono text-[var(--gray-1000)] relative z-50">
            <CarbonFabric muted />
            <div className="flex flex-col items-center z-10">
                <div className="w-12 h-12 border-t-2 border-white rounded-full animate-spin mb-6"></div>
                <div className="text-sm tracking-widest uppercase opacity-70">
                    Establishing Mesh Connection...
                </div>
            </div>
        </div>
    );
}
