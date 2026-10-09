import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class", '[data-theme="vault"]'],
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
        card: {
          DEFAULT: "var(--card)",
          foreground: "var(--card-foreground)",
        },
        popover: {
          DEFAULT: "var(--popover)",
          foreground: "var(--popover-foreground)",
        },
        primary: {
          DEFAULT: "var(--primary)",
          foreground: "var(--primary-foreground)",
        },
        secondary: {
          DEFAULT: "var(--secondary)",
          foreground: "var(--secondary-foreground)",
        },
        muted: {
          DEFAULT: "var(--muted)",
          foreground: "var(--muted-foreground)",
        },
        accent: {
          DEFAULT: "var(--accent)",
          foreground: "var(--accent-foreground)",
        },
        destructive: {
          DEFAULT: "var(--destructive)",
          foreground: "var(--destructive-foreground)",
        },
        border: "var(--border)",
        input: "var(--input)",
        ring: "var(--ring)",
        // CKODEX Direct Design System Tokens
        ck: {
          "bg-0": "var(--ck-bg-0)",
          "bg-1": "var(--ck-bg-1)",
          "bg-2": "var(--ck-bg-2)",
          "fg-1": "var(--ck-fg-1)",
          "fg-2": "var(--ck-fg-2)",
          "fg-3": "var(--ck-fg-3)",
          "fg-mute": "var(--ck-fg-mute)",
          accent: "var(--ck-accent)",
          hairline: "var(--ck-hairline)",
          "hairline-strong": "var(--ck-hairline-strong)",
          pos: "var(--ck-pos)",
          "pos-bg": "var(--ck-pos-bg)",
          neg: "var(--ck-neg)",
          "neg-bg": "var(--ck-neg-bg)",
          warn: "var(--ck-warn)",
          "warn-bg": "var(--ck-warn-bg)",
          info: "var(--ck-info)",
          "info-bg": "var(--ck-info-bg)",
          unk: "var(--ck-unk)",
          "unk-bg": "var(--ck-unk-bg)",
          "accent-text": "var(--ck-accent-text)",
        },
      },
      fontFamily: {
        serif: ["'Instrument Serif'", "Georgia", "serif"],
        mono: ["'JetBrains Mono Variable'", "ui-monospace", "monospace"],
        sans: ["'Inter Variable'", "system-ui", "-apple-system", "sans-serif"],
      },
      fontSize: {
        "2xs": ["11px", { lineHeight: "16px" }],
        xs: ["12px", { lineHeight: "16px" }],
        sm: ["13px", { lineHeight: "20px" }],
        base: ["14px", { lineHeight: "22px" }],
        md: ["16px", { lineHeight: "24px" }],
        lg: ["20px", { lineHeight: "28px" }],
        xl: ["28px", { lineHeight: "34px" }],
        "2xl": ["40px", { lineHeight: "44px" }],
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 1px)",
        sm: "calc(var(--radius) - 2px)",
      },
    },
  },
  plugins: [],
};

export default config;
