import DashboardClient from "./DashboardClient";

export const metadata = {
    title: "Mesh Dashboard | Triarchy",
    description: "Real-time view of the Casper Agentic Mesh",
};

export default function DashboardPage() {
    return (
        <main>
            <DashboardClient />
        </main>
    );
}
