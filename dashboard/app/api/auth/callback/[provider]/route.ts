import { NextRequest, NextResponse } from 'next/server';
import { AuthProvider } from '@/lib/auth/types';
import { verifyOAuthState, signUserSession } from '@/lib/auth/security';
import { exchangeCodeForUser } from '@/lib/auth/oauth';

export async function GET(
  request: NextRequest,
  { params }: { params: { provider: string } }
) {
  const provider = params.provider.toLowerCase() as AuthProvider;
  const origin = request.nextUrl.origin;
  const searchParams = request.nextUrl.searchParams;

  // Handle provider-level errors (e.g. user canceled consent)
  const oauthError = searchParams.get('error');
  if (oauthError) {
    const errorDesc =
      searchParams.get('error_description') || oauthError;
    return NextResponse.redirect(
      `${origin}/login?error=${encodeURIComponent(errorDesc)}`
    );
  }

  const code = searchParams.get('code');
  const queryState = searchParams.get('state');
  const isSandbox = searchParams.get('sandbox') === 'true';

  // Read stored CSRF state cookie
  const cookieState = request.cookies.get('dagr_oauth_state')?.value;

  // 1. Verify CSRF state token
  if (!queryState || !cookieState) {
    return NextResponse.redirect(
      `${origin}/login?error=${encodeURIComponent('Missing OAuth CSRF security state.')}`
    );
  }

  const verifiedState = verifyOAuthState(queryState, cookieState);
  if (!verifiedState) {
    return NextResponse.redirect(
      `${origin}/login?error=${encodeURIComponent('Invalid or expired CSRF state. Please try logging in again.')}`
    );
  }

  // 2. Exchange authorization code for user profile
  try {
    const user = await exchangeCodeForUser(
      provider,
      code || 'sandbox_code',
      origin,
      isSandbox
    );

    // 3. Cryptographically sign session token
    const sessionToken = signUserSession(user);
    const returnUrl = verifiedState.returnUrl || '/';
    const redirectTarget = returnUrl.startsWith('/')
      ? `${origin}${returnUrl}`
      : `${origin}/`;

    const response = NextResponse.redirect(redirectTarget);

    // 4. Set signed session cookie (30 days)
    response.cookies.set('dagr_session', sessionToken, {
      path: '/',
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 30 * 24 * 60 * 60, // 30 days
    });

    // 5. Clear one-time CSRF state cookie
    response.cookies.delete('dagr_oauth_state');

    return response;
  } catch (err: any) {
    return NextResponse.redirect(
      `${origin}/login?error=${encodeURIComponent(err.message || 'Token exchange failed.')}`
    );
  }
}
