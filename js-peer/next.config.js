/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  productionBrowserSourceMaps: true,
  images: {
    unoptimized: true,
  },
  // Configure for Replit environment
  async rewrites() {
    return []
  },
  // Allow all hosts for Replit proxy
  experimental: {
    allowedRevalidateHeaderKeys: [],
  },
}

module.exports = nextConfig
