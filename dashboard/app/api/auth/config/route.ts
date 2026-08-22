import { NextRequest, NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

const ENV_PATH = path.join(process.cwd(), '.env.local');

export async function GET(request: NextRequest) {
  return NextResponse.json({
    github: {
      configured: !!process.env.GITHUB_CLIENT_ID && !!process.env.GITHUB_CLIENT_SECRET,
      clientId: process.env.GITHUB_CLIENT_ID || '',
    },
    google: {
      configured: !!process.env.GOOGLE_CLIENT_ID && !!process.env.GOOGLE_CLIENT_SECRET,
      clientId: process.env.GOOGLE_CLIENT_ID || '',
    },
    microsoft: {
      configured: !!process.env.MICROSOFT_CLIENT_ID && !!process.env.MICROSOFT_CLIENT_SECRET,
      clientId: process.env.MICROSOFT_CLIENT_ID || '',
      tenantId: process.env.MICROSOFT_TENANT_ID || 'common',
    },
    email: {
      configured:
        !!process.env.RESEND_API_KEY ||
        (!!process.env.SMTP_HOST && !!process.env.SMTP_USER && !!process.env.SMTP_PASS),
      provider: process.env.RESEND_API_KEY ? 'resend' : process.env.SMTP_HOST ? 'smtp' : 'local_console',
      smtpHost: process.env.SMTP_HOST || '',
      smtpUser: process.env.SMTP_USER || '',
      fromEmail: process.env.EMAIL_FROM || '',
    },
  });
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const {
      githubClientId,
      githubClientSecret,
      googleClientId,
      googleClientSecret,
      microsoftClientId,
      microsoftClientSecret,
      microsoftTenantId,
      resendApiKey,
      smtpHost,
      smtpPort,
      smtpUser,
      smtpPass,
      emailFrom,
    } = body;

    // Update in-memory process.env
    if (githubClientId) process.env.GITHUB_CLIENT_ID = githubClientId.trim();
    if (githubClientSecret) process.env.GITHUB_CLIENT_SECRET = githubClientSecret.trim();
    if (googleClientId) process.env.GOOGLE_CLIENT_ID = googleClientId.trim();
    if (googleClientSecret) process.env.GOOGLE_CLIENT_SECRET = googleClientSecret.trim();
    if (microsoftClientId) process.env.MICROSOFT_CLIENT_ID = microsoftClientId.trim();
    if (microsoftClientSecret) process.env.MICROSOFT_CLIENT_SECRET = microsoftClientSecret.trim();
    if (microsoftTenantId) process.env.MICROSOFT_TENANT_ID = microsoftTenantId.trim();
    if (resendApiKey) process.env.RESEND_API_KEY = resendApiKey.trim();
    if (smtpHost) process.env.SMTP_HOST = smtpHost.trim();
    if (smtpPort) process.env.SMTP_PORT = smtpPort.trim();
    if (smtpUser) process.env.SMTP_USER = smtpUser.trim();
    if (smtpPass) process.env.SMTP_PASS = smtpPass.trim();
    if (emailFrom) process.env.EMAIL_FROM = emailFrom.trim();

    // Read existing .env.local or initialize
    let envContent = '';
    if (fs.existsSync(ENV_PATH)) {
      envContent = fs.readFileSync(ENV_PATH, 'utf-8');
    }

    const setEnvVar = (key: string, val: string | undefined) => {
      if (!val) return;
      const regex = new RegExp(`^${key}=.*$`, 'm');
      if (regex.test(envContent)) {
        envContent = envContent.replace(regex, `${key}=${val.trim()}`);
      } else {
        envContent += `\n${key}=${val.trim()}`;
      }
    };

    setEnvVar('GITHUB_CLIENT_ID', githubClientId);
    setEnvVar('GITHUB_CLIENT_SECRET', githubClientSecret);
    setEnvVar('GOOGLE_CLIENT_ID', googleClientId);
    setEnvVar('GOOGLE_CLIENT_SECRET', googleClientSecret);
    setEnvVar('MICROSOFT_CLIENT_ID', microsoftClientId);
    setEnvVar('MICROSOFT_CLIENT_SECRET', microsoftClientSecret);
    setEnvVar('MICROSOFT_TENANT_ID', microsoftTenantId);
    setEnvVar('RESEND_API_KEY', resendApiKey);
    setEnvVar('SMTP_HOST', smtpHost);
    setEnvVar('SMTP_PORT', smtpPort);
    setEnvVar('SMTP_USER', smtpUser);
    setEnvVar('SMTP_PASS', smtpPass);
    setEnvVar('EMAIL_FROM', emailFrom);

    fs.writeFileSync(ENV_PATH, envContent.trim() + '\n', 'utf-8');

    return NextResponse.json({
      success: true,
      message: 'OAuth & SMTP credentials updated and active.',
    });
  } catch (err: any) {
    return NextResponse.json(
      { error: err.message || 'Failed to save configuration' },
      { status: 500 }
    );
  }
}
