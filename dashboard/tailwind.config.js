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
        canvas: '#000000',
        obsidian: '#0D0E12',
        liquid: '#16171D',
        titaniumSlate: '#71717A',
        liquidPlatinum: '#E4E4E7',
        specular: 'rgba(255, 255, 255, 0.10)',
        specularStrong: 'rgba(255, 255, 255, 0.18)',
      },
      fontFamily: {
        brand: ['Space Grotesk', 'sans-serif'],
        sans: ['Geist', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
        mono: ['Geist Mono', 'JetBrains Mono', 'SF Mono', 'monospace'],
      },
    },
  },
  plugins: [],
}
