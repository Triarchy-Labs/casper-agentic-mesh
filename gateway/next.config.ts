import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* SSR mode — required for API Routes (/api/hire) to work */
  // The demo-tx route shells out to run_demo.sh, which needs the go-signer binary and the
  // disposable testnet key. Vercel's file tracer only sees the script's own literal path,
  // so the script's dependencies must be included explicitly or the hosted button can't sign.
  outputFileTracingIncludes: {
    "/api/demo-tx": [
      "../run_demo.sh",
      "../swarm/casper-client/key.pem",
      "../swarm/casper-client/go-signer/casper-tx-signer",
    ],
  },
  images: {
    formats: ['image/avif', 'image/webp'],
  },
};

export default nextConfig;
