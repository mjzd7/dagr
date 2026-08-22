'use client';
import React, { useEffect, useState } from 'react';
import Link from 'next/link';

interface AuthUser {
  id: string;
  email: string;
  name: string;
  avatar_url?: string;
  provider: string;
  org_name: string;
}

export function UserNav() {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function checkAuth() {
      try {
        const res = await fetch('/api/auth/me');
        if (res.ok) {
          const data = await res.json();
          if (data.authenticated && data.user) {
            setUser(data.user);
          }
        }
      } catch (e) {
        console.error('Failed to verify session', e);
      } finally {
        setLoading(false);
      }
    }
    checkAuth();
  }, []);

  const handleLogout = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      setUser(null);
      window.location.href = '/login';
    } catch (e) {
      window.location.href = '/login';
    }
  };

  if (loading) {
    return <div className="w-16 h-7 rounded-full bg-white/5 animate-pulse"></div>;
  }

  if (user) {
    return (
      <div className="flex items-center gap-3 text-xs font-mono">
        <div className="flex items-center gap-2 bg-white/5 border border-white/10 px-3 py-1.5 rounded-full">
          {user.avatar_url ? (
            <img
              src={user.avatar_url}
              alt={user.name}
              className="w-4 h-4 rounded-full object-cover border border-white/20"
            />
          ) : (
            <div className="w-4 h-4 rounded-full bg-white text-black font-bold flex items-center justify-center text-[9px] uppercase">
              {user.name.charAt(0)}
            </div>
          )}
          <span className="text-white font-medium truncate max-w-[120px]">
            {user.name}
          </span>
          <span className="text-zinc-500 capitalize">({user.provider})</span>
        </div>
        <button
          onClick={handleLogout}
          className="text-zinc-500 hover:text-white transition-colors text-xs"
          title="Sign Out"
        >
          Sign Out
        </button>
      </div>
    );
  }

  return (
    <Link
      href="/login"
      className="px-3.5 py-1.5 rounded-lg bg-white text-black text-xs font-semibold hover:bg-zinc-200 transition-colors shadow flex items-center gap-1.5"
    >
      <span>Sign In</span>
      <span className="text-[10px] opacity-60">→</span>
    </Link>
  );
}
