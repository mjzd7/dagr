'use client';
import React, { useState, useEffect, Suspense } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { BrandLogo, BrandLogoBadge } from '../../components/BrandLogo';

function LoginForm() {
  const searchParams = useSearchParams();
  const [errorMessage, setErrorMessage] = useState('');
  
  // Email Login State
  const [email, setEmail] = useState('');
  const [otpCode, setOtpCode] = useState('');
  const [verificationToken, setVerificationToken] = useState('');
  const [isOtpSent, setIsOtpSent] = useState(false);
  const [devCodeHint, setDevCodeHint] = useState('');
  const [emailLoading, setEmailLoading] = useState(false);
  const [emailStatus, setEmailStatus] = useState('');

  // API Key State
  const [apiKey, setApiKey] = useState('');
  const [orgName, setOrgName] = useState('');

  // OAuth Config Drawer & Live Status State
  const [showConfigModal, setShowConfigModal] = useState(false);
  const [authConfig, setAuthConfig] = useState<any>(null);
  const [configForm, setConfigForm] = useState({
    githubClientId: '',
    githubClientSecret: '',
    googleClientId: '',
    googleClientSecret: '',
    microsoftClientId: '',
    microsoftClientSecret: '',
    resendApiKey: '',
    smtpHost: '',
    smtpPort: '587',
    smtpUser: '',
    smtpPass: '',
    emailFrom: '',
  });
  const [configSaveStatus, setConfigSaveStatus] = useState('');

  useEffect(() => {
    const errorParam = searchParams.get('error');
    const setupParam = searchParams.get('setup');
    if (errorParam) {
      setErrorMessage(decodeURIComponent(errorParam));
    }
    if (setupParam) {
      setShowConfigModal(true);
    }
    fetchAuthConfig();
  }, [searchParams]);

  const fetchAuthConfig = async () => {
    try {
      const res = await fetch('/api/auth/config');
      if (res.ok) {
        const data = await res.json();
        setAuthConfig(data);
      }
    } catch (e) {}
  };

  const handleSaveConfig = async (e: React.FormEvent) => {
    e.preventDefault();
    setConfigSaveStatus('Saving credentials to .env.local...');
    try {
      const res = await fetch('/api/auth/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(configForm),
      });
      const data = await res.json();
      if (res.ok) {
        setConfigSaveStatus('✓ Live credentials saved to .env.local and activated!');
        fetchAuthConfig();
        setTimeout(() => {
          setShowConfigModal(false);
          setConfigSaveStatus('');
        }, 1200);
      } else {
        setConfigSaveStatus(`⚠️ ${data.error || 'Failed to save configuration'}`);
      }
    } catch (err: any) {
      setConfigSaveStatus(`⚠️ ${err.message || 'Network error'}`);
    }
  };

  // Handle Email OTP Request
  const handleRequestOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim()) return;

    setEmailLoading(true);
    setEmailStatus('');
    setErrorMessage('');

    try {
      const res = await fetch('/api/auth/email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'request_otp', email }),
      });

      const data = await res.json();
      if (!res.ok) {
        setErrorMessage(data.error || 'Failed to send OTP code');
        return;
      }

      setVerificationToken(data.verificationToken);
      setIsOtpSent(true);
      if (data.devCode) {
        setDevCodeHint(data.devCode);
      }
      setEmailStatus(data.message || `✓ 6-digit code sent to ${email}`);
    } catch (err: any) {
      setErrorMessage(err.message || 'Network error requesting OTP');
    } finally {
      setEmailLoading(false);
    }
  };

  // Handle Email OTP Verification
  const handleVerifyOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!otpCode.trim() || !verificationToken) return;

    setEmailLoading(true);
    setEmailStatus('');
    setErrorMessage('');

    try {
      const res = await fetch('/api/auth/email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'verify_otp',
          email,
          code: otpCode,
          verificationToken,
        }),
      });

      const data = await res.json();
      if (!res.ok) {
        setErrorMessage(data.error || 'Invalid or expired code');
        return;
      }

      setEmailStatus('✓ Verified! Redirecting to hypervisor...');
      window.location.href = '/';
    } catch (err: any) {
      setErrorMessage(err.message || 'Verification error');
    } finally {
      setEmailLoading(false);
    }
  };

  // Direct Organization API Key Login
  const handleApiKeyLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!apiKey.trim()) {
      setErrorMessage('Please enter an organization API key');
      return;
    }

    try {
      const res = await fetch('/api/auth/apikey', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ apiKey, orgName }),
      });

      if (!res.ok) {
        const data = await res.json();
        setErrorMessage(data.error || 'Failed to authenticate API key');
        return;
      }

      window.location.href = '/';
    } catch (err: any) {
      setErrorMessage(err.message || 'API key login failed');
    }
  };

  return (
    <div className="min-h-[85vh] flex flex-col items-center justify-center px-4 py-8 relative">
      <div className="w-full max-w-md space-y-6">
        
        {/* Brand Header */}
        <div className="text-center space-y-3 flex flex-col items-center">
          <BrandLogoBadge size={48} />
          <div>
            <h1 className="text-2xl font-bold font-brand tracking-tight text-white lowercase">
              dagr
            </h1>
            <p className="text-xs text-zinc-400 font-mono mt-1">
              Zero-Trust AST Slicing & FinOps Telemetry Plane
            </p>
          </div>
        </div>

        {/* Error Alert */}
        {errorMessage && (
          <div className="p-3.5 rounded-xl bg-red-500/10 border border-red-500/30 text-red-300 text-xs font-mono flex items-start gap-2.5">
            <span className="font-bold">⚠️</span>
            <div className="flex-1">{errorMessage}</div>
          </div>
        )}

        {/* Main Auth Glass Card */}
        <div className="p-7 rounded-3xl bg-zinc-950/90 border border-white/10 shadow-2xl space-y-5 backdrop-blur-xl">
          
          <div className="text-center">
            <h2 className="text-base font-bold text-white tracking-tight">
              Sign in to DAGR Control Plane
            </h2>
            <p className="text-xs text-zinc-500 font-mono mt-0.5">
              Production-grade OAuth 2.0 & Verified Email Sign-In
            </p>
          </div>

          {/* Social / OAuth Providers */}
          <div className="space-y-2.5">
            
            {/* GitHub OAuth Button */}
            <a
              href="/api/auth/github"
              className="w-full flex items-center justify-between px-4 py-2.5 rounded-xl bg-white hover:bg-zinc-200 text-black font-semibold text-xs transition shadow group"
            >
              <div className="flex items-center gap-3">
                <svg className="w-4 h-4 fill-current" viewBox="0 0 24 24">
                  <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
                </svg>
                <span>Continue with GitHub</span>
              </div>
              <span className={`text-[10px] font-mono px-2 py-0.5 rounded-full ${authConfig?.github?.configured ? 'bg-emerald-500/20 text-emerald-800 font-bold' : 'bg-black/10 text-zinc-600'}`}>
                {authConfig?.github?.configured ? 'LIVE' : 'SETUP'}
              </span>
            </a>

            {/* Google OAuth Button */}
            <a
              href="/api/auth/google"
              className="w-full flex items-center justify-between px-4 py-2.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-white font-semibold text-xs border border-white/10 transition group"
            >
              <div className="flex items-center gap-3">
                <svg className="w-4 h-4" viewBox="0 0 24 24">
                  <path fill="#4285F4" d="M23.745 12.27c0-.7-.06-1.4-.19-2.07H12v4.51h6.6c-.29 1.52-1.14 2.82-2.4 3.68v3.05h3.88c2.27-2.09 3.665-5.17 3.665-9.17z"/>
                  <path fill="#34A853" d="M12 24c3.24 0 5.95-1.08 7.93-2.91l-3.88-3.05c-1.08.72-2.45 1.16-4.05 1.16-3.12 0-5.77-2.1-6.72-4.93H1.26v3.15C3.25 21.36 7.34 24 12 24z"/>
                  <path fill="#FBBC05" d="M5.28 14.27c-.25-.72-.38-1.49-.38-2.27s.13-1.55.38-2.27V6.58H1.26C.46 8.16 0 9.94 0 12s.46 3.84 1.26 5.42l4.02-3.15z"/>
                  <path fill="#EA4335" d="M12 4.75c1.77 0 3.35.61 4.6 1.8l3.42-3.42C17.95 1.19 15.24 0 12 0 7.34 0 3.25 2.64 1.26 6.58l4.02 3.15c.95-2.83 3.6-4.98 6.72-4.98z"/>
                </svg>
                <span>Continue with Google</span>
              </div>
              <span className={`text-[10px] font-mono px-2 py-0.5 rounded-full ${authConfig?.google?.configured ? 'bg-emerald-500/20 text-emerald-400 font-bold' : 'bg-white/5 text-zinc-500'}`}>
                {authConfig?.google?.configured ? 'LIVE' : 'SETUP'}
              </span>
            </a>

            {/* Microsoft Entra ID OAuth Button */}
            <a
              href="/api/auth/microsoft"
              className="w-full flex items-center justify-between px-4 py-2.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-white font-semibold text-xs border border-white/10 transition group"
            >
              <div className="flex items-center gap-3">
                <svg className="w-4 h-4" viewBox="0 0 21 21">
                  <rect x="1" y="1" width="9" height="9" fill="#f25022"/>
                  <rect x="11" y="1" width="9" height="9" fill="#7fba00"/>
                  <rect x="1" y="11" width="9" height="9" fill="#00a4ef"/>
                  <rect x="11" y="11" width="9" height="9" fill="#ffb900"/>
                </svg>
                <span>Continue with Microsoft</span>
              </div>
              <span className={`text-[10px] font-mono px-2 py-0.5 rounded-full ${authConfig?.microsoft?.configured ? 'bg-emerald-500/20 text-emerald-400 font-bold' : 'bg-white/5 text-zinc-500'}`}>
                {authConfig?.microsoft?.configured ? 'LIVE' : 'SETUP'}
              </span>
            </a>

          </div>

          {/* Divider */}
          <div className="relative flex items-center justify-center">
            <div className="border-t border-white/10 w-full"></div>
            <span className="bg-zinc-950 px-3 text-[10px] font-mono text-zinc-500 uppercase tracking-widest absolute">
              or email sign-in
            </span>
          </div>

          {/* Passwordless Email OTP Login */}
          {!isOtpSent ? (
            <form onSubmit={handleRequestOtp} className="space-y-2.5 font-mono text-xs">
              <div>
                <div className="flex items-center justify-between text-zinc-400 mb-1">
                  <label>Corporate / Work Email</label>
                  <span className="text-[10px] text-zinc-500">
                    {authConfig?.email?.configured ? `Via ${authConfig.email.provider.toUpperCase()}` : 'Console / SMTP'}
                  </span>
                </div>
                <input
                  type="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="engineer@company.com"
                  className="w-full bg-black border border-white/15 rounded-lg px-3.5 py-2 text-white focus:outline-none focus:border-white"
                />
              </div>

              <button
                type="submit"
                disabled={emailLoading}
                className="w-full py-2.5 rounded-lg bg-white/10 hover:bg-white/20 text-white font-semibold transition border border-white/15 text-xs"
              >
                {emailLoading ? 'Dispatching OTP...' : 'Send 6-Digit Code →'}
              </button>
            </form>
          ) : (
            <form onSubmit={handleVerifyOtp} className="space-y-2.5 font-mono text-xs">
              <div className="flex items-center justify-between text-zinc-400">
                <span>Enter 6-Digit Code sent to:</span>
                <button
                  type="button"
                  onClick={() => setIsOtpSent(false)}
                  className="text-white hover:underline text-[11px]"
                >
                  Change Email
                </button>
              </div>
              <div className="text-zinc-300 font-semibold truncate">{email}</div>

              {devCodeHint && (
                <div className="p-2 rounded bg-white/5 border border-white/10 text-[11px] text-emerald-400">
                  ⚡ Local Code: <span className="font-bold tracking-widest">{devCodeHint}</span>
                </div>
              )}

              <input
                type="text"
                required
                maxLength={6}
                value={otpCode}
                onChange={(e) => setOtpCode(e.target.value)}
                placeholder="123456"
                className="w-full bg-black border border-white/20 rounded-lg px-3.5 py-2.5 text-center text-lg tracking-widest font-mono text-white focus:outline-none focus:border-white"
              />

              <button
                type="submit"
                disabled={emailLoading}
                className="w-full py-2.5 rounded-lg bg-white text-black font-semibold transition text-xs shadow hover:bg-zinc-200"
              >
                {emailLoading ? 'Verifying...' : 'Verify Code & Sign In'}
              </button>
            </form>
          )}

          {emailStatus && (
            <div className="p-2.5 rounded-lg bg-white/5 border border-white/10 text-center font-mono text-xs text-white">
              {emailStatus}
            </div>
          )}

          {/* Divider */}
          <div className="relative flex items-center justify-center pt-2">
            <div className="border-t border-white/10 w-full"></div>
            <span className="bg-zinc-950 px-3 text-[10px] font-mono text-zinc-500 uppercase tracking-widest absolute">
              or node API key
            </span>
          </div>

          {/* API Key Section */}
          <form onSubmit={handleApiKeyLogin} className="space-y-2.5 font-mono text-xs">
            <div className="grid grid-cols-2 gap-2">
              <input
                type="text"
                value={orgName}
                onChange={(e) => setOrgName(e.target.value)}
                placeholder="Org (Acme Corp)"
                className="w-full bg-black border border-white/15 rounded-lg px-3 py-2 text-white text-xs focus:outline-none focus:border-white"
              />
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="dagr_live_sec_..."
                className="w-full bg-black border border-white/15 rounded-lg px-3 py-2 text-white text-xs focus:outline-none focus:border-white"
              />
            </div>

            <button
              type="submit"
              className="w-full py-2 rounded-lg bg-zinc-900 hover:bg-zinc-800 text-zinc-300 font-semibold transition border border-white/10 text-xs"
            >
              Sign In with API Key
            </button>
          </form>

        </div>

        {/* Bottom Utility Actions */}
        <div className="flex items-center justify-between text-xs font-mono text-zinc-500 px-2">
          <Link href="/" className="hover:text-zinc-300 transition-colors">
            ← Return to Dashboard
          </Link>
          <button
            onClick={() => setShowConfigModal(true)}
            className="hover:text-white transition-colors flex items-center gap-1 text-zinc-400"
          >
            <span>⚙️</span>
            <span>Configure OAuth & SMTP</span>
          </button>
        </div>

      </div>

      {/* Interactive OAuth & SMTP Configuration Modal */}
      {showConfigModal && (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4 overflow-y-auto">
          <div className="w-full max-w-2xl bg-zinc-950 border border-white/15 rounded-3xl p-6 shadow-2xl space-y-6 max-h-[90vh] overflow-y-auto">
            
            <div className="flex items-center justify-between border-b border-white/10 pb-4">
              <div>
                <h3 className="text-base font-bold text-white tracking-tight">
                  ⚙️ Live OAuth & SMTP Provider Configuration
                </h3>
                <p className="text-xs text-zinc-400 font-mono mt-0.5">
                  Enter your official production credentials to enable live authorization & email dispatch
                </p>
              </div>
              <button
                onClick={() => setShowConfigModal(false)}
                className="text-zinc-400 hover:text-white text-sm font-mono px-2 py-1 rounded-lg bg-white/5"
              >
                ✕ Close
              </button>
            </div>

            <form onSubmit={handleSaveConfig} className="space-y-6 text-xs font-mono">
              
              {/* GitHub OAuth Setup */}
              <div className="p-4 rounded-2xl bg-black border border-white/10 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-white text-sm flex items-center gap-2">
                    <span>🐙 GitHub OAuth</span>
                    {authConfig?.github?.configured && <span className="text-emerald-400 text-[10px]">● Active</span>}
                  </span>
                  <a
                    href="https://github.com/settings/developers"
                    target="_blank"
                    rel="noreferrer"
                    className="text-zinc-400 hover:text-white underline text-[11px]"
                  >
                    Create OAuth App in GitHub Settings ↗
                  </a>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <label className="block text-zinc-400 mb-1">GITHUB_CLIENT_ID</label>
                    <input
                      type="text"
                      placeholder="e.g. Ov23li..."
                      value={configForm.githubClientId}
                      onChange={(e) => setConfigForm({ ...configForm, githubClientId: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                  <div>
                    <label className="block text-zinc-400 mb-1">GITHUB_CLIENT_SECRET</label>
                    <input
                      type="password"
                      placeholder="e.g. 7f9a8b..."
                      value={configForm.githubClientSecret}
                      onChange={(e) => setConfigForm({ ...configForm, githubClientSecret: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                </div>
                <p className="text-[10px] text-zinc-500">Callback URL: <code className="text-zinc-400">{typeof window !== 'undefined' ? window.location.origin : ''}/api/auth/callback/github</code></p>
              </div>

              {/* Google OAuth Setup */}
              <div className="p-4 rounded-2xl bg-black border border-white/10 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-white text-sm flex items-center gap-2">
                    <span>🌐 Google Cloud OAuth 2.0</span>
                    {authConfig?.google?.configured && <span className="text-emerald-400 text-[10px]">● Active</span>}
                  </span>
                  <a
                    href="https://console.cloud.google.com/apis/credentials"
                    target="_blank"
                    rel="noreferrer"
                    className="text-zinc-400 hover:text-white underline text-[11px]"
                  >
                    Google Cloud Console ↗
                  </a>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <label className="block text-zinc-400 mb-1">GOOGLE_CLIENT_ID</label>
                    <input
                      type="text"
                      placeholder="e.g. 109823...apps.googleusercontent.com"
                      value={configForm.googleClientId}
                      onChange={(e) => setConfigForm({ ...configForm, googleClientId: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                  <div>
                    <label className="block text-zinc-400 mb-1">GOOGLE_CLIENT_SECRET</label>
                    <input
                      type="password"
                      placeholder="GOCSPX-..."
                      value={configForm.googleClientSecret}
                      onChange={(e) => setConfigForm({ ...configForm, googleClientSecret: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                </div>
                <p className="text-[10px] text-zinc-500">Redirect URI: <code className="text-zinc-400">{typeof window !== 'undefined' ? window.location.origin : ''}/api/auth/callback/google</code></p>
              </div>

              {/* Microsoft Entra ID Setup */}
              <div className="p-4 rounded-2xl bg-black border border-white/10 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-white text-sm flex items-center gap-2">
                    <span>🪟 Microsoft Entra ID</span>
                    {authConfig?.microsoft?.configured && <span className="text-emerald-400 text-[10px]">● Active</span>}
                  </span>
                  <a
                    href="https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade"
                    target="_blank"
                    rel="noreferrer"
                    className="text-zinc-400 hover:text-white underline text-[11px]"
                  >
                    Azure App Registrations ↗
                  </a>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <label className="block text-zinc-400 mb-1">MICROSOFT_CLIENT_ID</label>
                    <input
                      type="text"
                      placeholder="e.g. e4f5a6b7-..."
                      value={configForm.microsoftClientId}
                      onChange={(e) => setConfigForm({ ...configForm, microsoftClientId: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                  <div>
                    <label className="block text-zinc-400 mb-1">MICROSOFT_CLIENT_SECRET</label>
                    <input
                      type="password"
                      placeholder="e.g. ~aB3q..."
                      value={configForm.microsoftClientSecret}
                      onChange={(e) => setConfigForm({ ...configForm, microsoftClientSecret: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                </div>
                <p className="text-[10px] text-zinc-500">Redirect URI: <code className="text-zinc-400">{typeof window !== 'undefined' ? window.location.origin : ''}/api/auth/callback/microsoft</code></p>
              </div>

              {/* SMTP / Resend Email Setup */}
              <div className="p-4 rounded-2xl bg-black border border-white/10 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-white text-sm flex items-center gap-2">
                    <span>✉️ Production Email (SMTP / Resend)</span>
                    {authConfig?.email?.configured && <span className="text-emerald-400 text-[10px]">● Active ({authConfig.email.provider})</span>}
                  </span>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <label className="block text-zinc-400 mb-1">RESEND_API_KEY (Recommended)</label>
                    <input
                      type="password"
                      placeholder="re_123456..."
                      value={configForm.resendApiKey}
                      onChange={(e) => setConfigForm({ ...configForm, resendApiKey: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                  <div>
                    <label className="block text-zinc-400 mb-1">SMTP_HOST (or Gmail/Outlook/SES)</label>
                    <input
                      type="text"
                      placeholder="smtp.gmail.com or smtp.sendgrid.net"
                      value={configForm.smtpHost}
                      onChange={(e) => setConfigForm({ ...configForm, smtpHost: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                  <div>
                    <label className="block text-zinc-400 mb-1">SMTP_USER</label>
                    <input
                      type="text"
                      placeholder="user@company.com"
                      value={configForm.smtpUser}
                      onChange={(e) => setConfigForm({ ...configForm, smtpUser: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                  <div>
                    <label className="block text-zinc-400 mb-1">SMTP_PASS</label>
                    <input
                      type="password"
                      placeholder="App Password or SMTP API Key"
                      value={configForm.smtpPass}
                      onChange={(e) => setConfigForm({ ...configForm, smtpPass: e.target.value })}
                      className="w-full bg-zinc-950 border border-white/15 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-white"
                    />
                  </div>
                </div>
              </div>

              {configSaveStatus && (
                <div className="p-3 rounded-lg bg-white/10 border border-white/20 text-center text-white">
                  {configSaveStatus}
                </div>
              )}

              <div className="flex items-center justify-end gap-3 pt-2">
                <button
                  type="button"
                  onClick={() => setShowConfigModal(false)}
                  className="px-4 py-2.5 rounded-xl bg-zinc-900 hover:bg-zinc-800 text-zinc-300 font-semibold"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-6 py-2.5 rounded-xl bg-white hover:bg-zinc-200 text-black font-semibold shadow"
                >
                  Save & Activate Credentials
                </button>
              </div>

            </form>

          </div>
        </div>
      )}

    </div>
  );
}

export default function LoginPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-[85vh] flex items-center justify-center">
          <div className="w-8 h-8 rounded-full border-2 border-white/20 border-t-white animate-spin"></div>
        </div>
      }
    >
      <LoginForm />
    </Suspense>
  );
}
