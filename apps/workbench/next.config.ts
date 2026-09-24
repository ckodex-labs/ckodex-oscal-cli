import type { NextConfig } from "next";

const isExport = process.env.NEXT_OUTPUT_EXPORT === "true";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  transpilePackages: ["lucide-react"],
  devIndicators: false,
  ...(isExport
    ? {
        output: "export",
        basePath: "/ckodex-oscal-cli",
        trailingSlash: true,
      }
    : {}),
};

export default nextConfig;
