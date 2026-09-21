import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Mizan · OSCAL Compliance Workbench",
  description:
    "High-Assurance OSCAL Graph of Record, Multi-Lens Explorer & AI Copilot",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased font-mono">{children}</body>
    </html>
  );
}
