/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#090d16',
        surface: '#0f172a',
        surfaceBorder: '#1e293b',
        dagrCyan: '#06b6d4',
        dagrGreen: '#10b981',
        dagrPurple: '#8b5cf6',
      },
    },
  },
  plugins: [],
}
