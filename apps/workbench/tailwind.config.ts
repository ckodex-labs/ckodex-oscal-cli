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
        },
      },
      fontFamily: {
        serif: ["'Instrument Serif'", "Georgia", "serif"],
        mono: ["'JetBrains Mono'", "monospace"],
        sans: ["system-ui", "-apple-system", "sans-serif"],
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
