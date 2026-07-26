import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* SSR mode — required for API Routes (/api/hire) to work */
  images: {
    formats: ['image/avif', 'image/webp'],
  },
};

export default nextConfig;
