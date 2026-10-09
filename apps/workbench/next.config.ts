import type { NextConfig } from "next";

const isExport = process.env.NEXT_OUTPUT_EXPORT === "true";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  transpilePackages: ["lucide-react"],
  devIndicators: false,
  env: {
    NEXT_PUBLIC_BASE_PATH: isExport ? "/ckodex-oscal-cli" : "",
  },
  ...(isExport
    ? {
        output: "export",
        basePath: "/ckodex-oscal-cli",
        trailingSlash: true,
      }
    : {}),
};

export default nextConfig;
