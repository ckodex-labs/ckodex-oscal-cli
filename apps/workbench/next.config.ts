import type { NextConfig } from "next";

const isExport = process.env.NEXT_OUTPUT_EXPORT === "true";
const exportBasePath =
  process.env.NEXT_EXPORT_BASE_PATH !== undefined
    ? process.env.NEXT_EXPORT_BASE_PATH
    : isExport
      ? "/ckodex-oscal-cli"
      : "";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  transpilePackages: ["lucide-react"],
  devIndicators: false,
  env: {
    NEXT_PUBLIC_BASE_PATH: exportBasePath,
    NEXT_PUBLIC_STATIC_EXPORT: isExport ? "true" : "false",
  },
  ...(isExport
    ? {
        output: "export",
        ...(exportBasePath ? { basePath: exportBasePath } : {}),
        trailingSlash: true,
      }
    : {}),
};

export default nextConfig;
