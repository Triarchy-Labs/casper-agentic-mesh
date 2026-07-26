import React from "react";
import { Nav } from "@/components/Nav";

export default function AppLayout({ children }: { children: React.ReactNode }) {
    return (
        <div className="app-layout relative min-h-screen">
            <Nav />
            {children}
        </div>
    );
}
