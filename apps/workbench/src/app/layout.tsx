import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Mizan · OSCAL Compliance Workbench",
  description:
    "OSCAL compliance workbench for the Mizan engine. Every panel states where its data came from.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" data-theme="ledger">
      <body className="antialiased">{children}</body>
    </html>
  );
}
