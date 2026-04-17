import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "#f6f4ef",
        foreground: "#172033",
        card: "#fffdf7",
        border: "#d5d0bf",
        primary: "#1e5b52",
        accent: "#e8a54b",
        muted: "#ede8da",
      },
      borderRadius: {
        xl: "1rem",
      },
      boxShadow: {
        panel: "0 16px 48px rgba(23, 32, 51, 0.08)",
      },
    },
  },
  plugins: [],
} satisfies Config;
