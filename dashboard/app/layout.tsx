import './globals.css';
import React from 'react';
import { BrandLogo, BrandLogoBadge } from '../components/BrandLogo';
import { UserNav } from '../components/UserNav';
import Link from 'next/link';

export const metadata = {
  title: 'dagr // 3D AST Knowledge Graph & Control Plane',
  description: 'Real-time telemetry, 3D AST knowledge graph, and proof-of-correctness control plane for AI coding agents.',
  icons: {
    icon: '/favicon.svg',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <head>
        <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700;800;900&family=Geist+Mono:wght@400;500;600;700&family=Space+Grotesk:wght@600;700&display=swap" rel="stylesheet" />
      </head>
      <body className="min-h-screen bg-black text-[#E4E4E7] flex flex-col font-sans antialiased selection:bg-white selection:text-black">
        <header className="border-b border-white/10 bg-black/90 backdrop-blur-xl sticky top-0 z-50 px-6 py-4 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-3 group">
            {/* Official Tri-Node Euclidean DAG Logo Badge */}
            <BrandLogoBadge size={32} />
            <div>
              <div className="font-bold text-lg tracking-tight font-brand text-white lowercase group-hover:text-zinc-200 transition">
                dagr
              </div>
              <div className="text-xs text-zinc-500 font-mono">v0.1.0 • Control Plane</div>
            </div>
          </Link>
          
          <div className="flex items-center gap-4 text-xs font-mono">
            <div className="hidden sm:flex items-center gap-2 bg-white/5 border border-white/10 text-white px-3 py-1.5 rounded-full">
              <span className="h-2 w-2 rounded-full bg-white animate-pulse"></span>
              CDC WAL: Active
            </div>
            <div className="hidden sm:flex items-center gap-2 bg-white/5 border border-white/10 text-zinc-300 px-3 py-1.5 rounded-full">
              <span className="h-2 w-2 rounded-full bg-emerald-400"></span>
              Memgraph Bolt: Connected
            </div>
            {/* User Session & Login Button */}
            <UserNav />
          </div>
        </header>
        
        <main className="flex-1 p-6 max-w-7xl mx-auto w-full">{children}</main>
        
        <footer className="border-t border-white/10 py-4 px-6 text-xs font-mono text-zinc-500 flex justify-between items-center bg-black">
          <span className="font-brand font-bold text-zinc-400">dagr hypervisor</span>
          <span>Zero-PII • Deterministic Program Slicing</span>
        </footer>
      </body>
    </html>
  );
}
