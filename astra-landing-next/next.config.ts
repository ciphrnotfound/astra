import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  typescript: {
    // Skip TypeScript checks during build
    ignoreBuildErrors: true,
  },
};

export default nextConfig;
