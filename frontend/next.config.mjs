/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  images: { unoptimized: true },
  async rewrites() {
    return [
      { source: '/api/:path*', destination: 'http://localhost:12234/api/:path*' },
      { source: '/mcp', destination: 'http://localhost:12234/mcp' },
      { source: '/health', destination: 'http://localhost:12234/health' },
    ];
  },
};

export default nextConfig;
