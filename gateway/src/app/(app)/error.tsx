"use client";

import React, { useEffect } from "react";
import { CarbonFabric } from "@/components/CarbonFabric";

export default function AppError({
    error,
    reset,
}: {
    error: Error & { digest?: string };
    reset: () => void;
}) {
    useEffect(() => {
        console.error("App Boundary Caught Error:", error);
    }, [error]);

    return (
        <div className="min-h-screen flex flex-col items-center justify-center font-mono text-[var(--gray-1000)] relative z-50">
            <CarbonFabric muted />
            <div className="z-10 text-center p-8 border border-red-500/20 bg-red-900/10 backdrop-blur-md rounded-xl max-w-2xl">
                <h2 className="text-2xl text-red-400 mb-4 tracking-tighter">
                    [SYSTEM_FAULT] UI Thread Unstable
                </h2>
                <p className="mb-6 opacity-70">
                    A severe client-side error occurred in the interactive Mesh interface.
                    <br/>
                    <span className="text-xs font-mono opacity-50 block mt-2">{error.message}</span>
                </p>
                <button
                    onClick={() => reset()}
                    className="px-6 py-2 border border-[var(--gray-700)] hover:border-white transition-colors uppercase text-sm"
                >
                    Re-initialize Subsystem
                </button>
            </div>
        </div>
    );
}
