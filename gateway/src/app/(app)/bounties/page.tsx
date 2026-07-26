import BountiesClient from "./BountiesClient";

export const metadata = {
    title: "Bounty Board | Triarchy",
    description: "Sovereign Bounty Board - Escrow, proofs, zero-trust.",
};

export default function BountiesPage() {
    return (
        <main>
            <BountiesClient />
        </main>
    );
}
