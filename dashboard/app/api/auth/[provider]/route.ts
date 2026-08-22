import { NextRequest, NextResponse } from 'next/server';
import { AuthProvider, AuthUser } from '@/lib/auth/types';
import {
  generateOAuthState,
  generateEmailOtp,
  verifyEmailOtp,
  signUserSession,
} from '@/lib/auth/security';
import { buildProviderAuthUrl } from '@/lib/auth/oauth';
import { sendVerificationEmail } from '@/lib/auth/email';

export async function GET(
  request: NextRequest,
  { params }: { params: { provider: string } }
) {
  const provider = params.provider.toLowerCase() as AuthProvider;
  const origin = request.nextUrl.origin;
  const returnUrl = request.nextUrl.searchParams.get('returnUrl') || '/';

  if (!['github', 'google', 'microsoft'].includes(provider)) {
    return NextResponse.json(
      { error: `Invalid OAuth provider: ${params.provider}` },
      { status: 400 }
    );
  }

  // 1. Generate cryptographically signed CSRF state
  const { stateToken, cookieValue } = generateOAuthState(
    provider,
    origin,
    returnUrl
  );

  // 2. Build official authorization URL
  try {
    const { url } = buildProviderAuthUrl(provider, origin, stateToken);
    const response = NextResponse.redirect(url);

    // 3. Set secure, httpOnly CSRF cookie (10 minute expiry)
    response.cookies.set('dagr_oauth_state', cookieValue, {
      path: '/',
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 10 * 60, // 10 minutes
    });

    return response;
  } catch (err: any) {
    return NextResponse.redirect(
      `${origin}/login?error=${encodeURIComponent(err.message || 'auth_init_failed')}`
    );
  }
}

/**
 * Handles Email authentication (Send OTP or Verify OTP)
 */
export async function POST(
  request: NextRequest,
  { params }: { params: { provider: string } }
) {
  const provider = params.provider.toLowerCase();

  // Support direct API Key authentication
  if (provider === 'apikey') {
    try {
      const body = await request.json();
      const apiKey = (body.apiKey || '').trim();
      const orgName = (body.orgName || 'Default Organization').trim();

      if (!apiKey) {
        return NextResponse.json(
          { error: 'API Key is required' },
          { status: 400 }
        );
      }

      const user: AuthUser = {
        id: `key:${apiKey.substring(0, 12)}`,
        email: `admin@${orgName.toLowerCase().replace(/[^a-z0-9]/g, '')}.internal`,
        name: `${orgName} Admin`,
        provider: 'apikey',
        org_id: `org_${orgName.toLowerCase().replace(/[^a-z0-9]/g, '_')}`,
        org_name: orgName,
        created_at: Date.now(),
      };

      const sessionToken = signUserSession(user);
      const response = NextResponse.json({ success: true, user });

      response.cookies.set('dagr_session', sessionToken, {
        path: '/',
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 30 * 24 * 60 * 60, // 30 days
      });

      return response;
    } catch (err: any) {
      return NextResponse.json(
        { error: err.message || 'API key auth error' },
        { status: 500 }
      );
    }
  }

  if (provider !== 'email') {
    return NextResponse.json(
      { error: 'Endpoint only supports email and apikey authentication' },
      { status: 400 }
    );
  }

  try {
    const body = await request.json();
    const action = body.action; // 'request_otp' or 'verify_otp'
    const email = (body.email || '').trim().toLowerCase();

    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      return NextResponse.json(
        { error: 'Please provide a valid corporate or personal email address.' },
        { status: 400 }
      );
    }

    // Step 1: Request 6-digit OTP
    if (action === 'request_otp') {
      const { code, verificationToken } = generateEmailOtp(email);

      // Dispatch real email via SMTP or Resend
      const emailResult = await sendVerificationEmail(email, code);

      return NextResponse.json({
        success: true,
        message: emailResult.sent
          ? `Verification email delivered to ${email}.`
          : `Verification code generated for ${email}.`,
        verificationToken,
        provider: emailResult.provider,
        devCode: !emailResult.sent ? code : undefined,
      });
    }

    // Step 2: Verify 6-digit OTP
    if (action === 'verify_otp') {
      const { code, verificationToken } = body;
      if (!code || !verificationToken) {
        return NextResponse.json(
          { error: 'Missing verification code or token.' },
          { status: 400 }
        );
      }

      const isValid = verifyEmailOtp(verificationToken, code, email);
      if (!isValid) {
        return NextResponse.json(
          { error: 'Invalid or expired verification code.' },
          { status: 401 }
        );
      }

      // Generate verified session
      const domain = email.split('@')[1];
      const orgName = domain.replace(/\.[^.]+$/, '');
      const user = {
        id: `email:${email}`,
        email,
        name: email.split('@')[0],
        provider: 'email' as AuthProvider,
        org_id: `org_${orgName.toLowerCase().replace(/[^a-z0-9]/g, '_')}`,
        org_name: orgName.toUpperCase(),
        created_at: Date.now(),
      };

      const sessionToken = signUserSession(user);
      const response = NextResponse.json({ success: true, user });

      // Set signed session cookie
      response.cookies.set('dagr_session', sessionToken, {
        path: '/',
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 30 * 24 * 60 * 60, // 30 days
      });

      return response;
    }

    return NextResponse.json({ error: 'Invalid action' }, { status: 400 });
  } catch (err: any) {
    return NextResponse.json(
      { error: err.message || 'Email auth error' },
      { status: 500 }
    );
  }
}
