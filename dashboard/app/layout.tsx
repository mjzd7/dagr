import './globals.css';
import React from 'react';

export const metadata = {
  title: '⚡ DAGR — 3D AST Knowledge Graph & Hypervisor Control Plane',
  description: 'Real-time telemetry, 3D AST knowledge graph, and proof-of-correctness control plane for AI coding agents.',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-[#090d16] text-slate-100 flex flex-col antialiased">
        <header className="border-b border-slate-800/80 bg-[#0f172a]/70 backdrop-blur-md sticky top-0 z-50 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-lg bg-gradient-to-tr from-cyan-500 to-indigo-600 flex items-center justify-center font-black text-white text-lg shadow-lg shadow-cyan-500/20">
              ⚡
            </div>
            <div>
              <div className="font-bold text-lg tracking-tight bg-gradient-to-r from-white via-slate-200 to-cyan-400 bg-clip-text text-transparent">
                DAGR Hypervisor
              </div>
              <div className="text-xs text-slate-400 font-mono">v0.1.0 • Cold Plane Node: #01</div>
            </div>
          </div>
          <div className="flex items-center gap-4 text-xs font-mono">
            <div className="flex items-center gap-2 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 px-3 py-1.5 rounded-full">
              <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
              Redpanda CDC: Active
            </div>
            <div className="flex items-center gap-2 bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 px-3 py-1.5 rounded-full">
              <span className="h-2 w-2 rounded-full bg-cyan-400 animate-pulse"></span>
              Memgraph Bolt: Connected
            </div>
          </div>
        </header>
        <main className="flex-1 p-6 max-w-7xl mx-auto w-full">{children}</main>
      </body>
    </html>
  );
}
