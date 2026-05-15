/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        surface: {
          0: '#18181b',
          1: '#1c1c1f',
          2: '#242427',
          3: '#2a2a2e',
        },
        accent: {
          DEFAULT: '#3b82f6',
          hover: '#60a5fa',
          muted: '#3b82f620',
        },
        txt: {
          primary: '#e4e4e7',
          secondary: '#71717a',
          muted: '#52525b',
        },
        bdr: {
          DEFAULT: '#27272a',
          hover: '#3f3f46',
        },
        ok: '#22c55e',
        err: '#ef4444',
        warn: '#f59e0b',
      },
    },
  },
  plugins: [],
}
