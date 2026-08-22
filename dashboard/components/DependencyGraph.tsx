'use client';
import React, { useState } from 'react';

export function DependencyGraph() {
  const [selectedNode, setSelectedNode] = useState('processPayment');

  return (
    <div className="p-6 rounded-2xl bg-zinc-950/80 border border-white/10 shadow-2xl relative">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-base font-bold text-white flex items-center gap-2">
            3D Blast Radius & AST Dependency Graph
          </h2>
          <p className="text-xs text-zinc-400 font-mono mt-0.5">Memgraph Bolt Query: 2-Hop APOC Transitive Closure</p>
        </div>
        <div className="flex items-center gap-2 text-xs font-mono">
          <span className="px-2.5 py-1 rounded-md bg-zinc-900 border border-white/10 text-zinc-300">Seed: {selectedNode}</span>
        </div>
      </div>

      <div className="h-72 w-full rounded-xl bg-black border border-white/10 flex items-center justify-center relative overflow-hidden p-6">
        <div className="absolute inset-0 bg-[radial-gradient(#27272a_1px,transparent_1px)] [background-size:16px_16px] opacity-40"></div>
        
        {/* Visual Graph Nodes */}
        <div className="grid grid-cols-3 gap-8 w-full max-w-xl z-10">
          <div className="col-span-1 flex flex-col gap-4">
            <div 
              onClick={() => setSelectedNode('PaymentIntent')}
              className="p-3 rounded-lg border border-white/15 bg-zinc-950/80 text-zinc-200 text-xs font-mono cursor-pointer hover:border-white/40 transition"
            >
              <div className="text-[10px] text-zinc-500">INTERFACE</div>
              <div className="font-bold">PaymentIntent</div>
            </div>
            <div 
              onClick={() => setSelectedNode('PaymentResult')}
              className="p-3 rounded-lg border border-white/15 bg-zinc-950/80 text-zinc-200 text-xs font-mono cursor-pointer hover:border-white/40 transition"
            >
              <div className="text-[10px] text-zinc-500">INTERFACE</div>
              <div className="font-bold">PaymentResult</div>
            </div>
          </div>

          <div className="col-span-1 flex items-center justify-center">
            <div 
              onClick={() => setSelectedNode('processPayment')}
              className="p-4 rounded-xl border-2 border-white bg-white/10 text-white text-center font-mono cursor-pointer shadow-lg shadow-white/10 scale-105"
            >
              <div className="text-[10px] text-zinc-400 font-bold">TARGET SYMBOL</div>
              <div className="font-bold text-sm">processPayment</div>
              <div className="text-[10px] text-zinc-400 mt-1">34 lines • 342 tokens</div>
            </div>
          </div>

          <div className="col-span-1 flex flex-col gap-4">
            <div 
              onClick={() => setSelectedNode('StripeGateway')}
              className="p-3 rounded-lg border border-white/15 bg-zinc-950/80 text-zinc-200 text-xs font-mono cursor-pointer hover:border-white/40 transition"
            >
              <div className="text-[10px] text-zinc-500">CALLS</div>
              <div className="font-bold">StripeGateway</div>
            </div>
            <div 
              onClick={() => setSelectedNode('dbClient')}
              className="p-3 rounded-lg border border-white/15 bg-zinc-950/80 text-zinc-400 text-xs font-mono cursor-pointer hover:border-white/40 transition"
            >
              <div className="text-[10px] text-zinc-600">BLOCKED BOUNDARY</div>
              <div className="font-bold">@db/client</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
