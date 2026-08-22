import { AuthProvider, AuthUser } from './types';

export interface ProviderAuthUrlResult {
  url: string;
  isMock: boolean;
}

/**
 * Builds the provider authorization consent URL
 */
export function buildProviderAuthUrl(
  provider: AuthProvider,
  origin: string,
  state: string
): ProviderAuthUrlResult {
  const redirectUri = `${origin}/api/auth/callback/${provider}`;

  if (provider === 'github') {
    const clientId = process.env.GITHUB_CLIENT_ID;
    if (!clientId) {
      return {
        url: `${origin}/login?setup=github&error=${encodeURIComponent('GitHub OAuth requires GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET. Click Configure Keys below.')}`,
        isMock: true,
      };
    }
    const params = new URLSearchParams({
      client_id: clientId,
      redirect_uri: redirectUri,
      scope: 'read:user,user:email',
      state: state,
    });
    return {
      url: `https://github.com/login/oauth/authorize?${params.toString()}`,
      isMock: false,
    };
  }

  if (provider === 'google') {
    const clientId = process.env.GOOGLE_CLIENT_ID;
    if (!clientId) {
      return {
        url: `${origin}/login?setup=google&error=${encodeURIComponent('Google OAuth requires GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET. Click Configure Keys below.')}`,
        isMock: true,
      };
    }
    const params = new URLSearchParams({
      client_id: clientId,
      redirect_uri: redirectUri,
      response_type: 'code',
      scope: 'openid profile email',
      access_type: 'offline',
      prompt: 'consent',
      state: state,
    });
    return {
      url: `https://accounts.google.com/o/oauth2/v2/auth?${params.toString()}`,
      isMock: false,
    };
  }

  if (provider === 'microsoft') {
    const clientId = process.env.MICROSOFT_CLIENT_ID;
    const tenant = process.env.MICROSOFT_TENANT_ID || 'common';
    if (!clientId) {
      return {
        url: `${origin}/login?setup=microsoft&error=${encodeURIComponent('Microsoft OAuth requires MICROSOFT_CLIENT_ID and MICROSOFT_CLIENT_SECRET. Click Configure Keys below.')}`,
        isMock: true,
      };
    }
    const params = new URLSearchParams({
      client_id: clientId,
      redirect_uri: redirectUri,
      response_type: 'code',
      scope: 'openid profile email User.Read',
      response_mode: 'query',
      state: state,
    });
    return {
      url: `https://login.microsoftonline.com/${tenant}/oauth2/v2.0/authorize?${params.toString()}`,
      isMock: false,
    };
  }

  throw new Error(`Unsupported OAuth provider: ${provider}`);
}

/**
 * Exchanges authorization code for normalized AuthUser profile
 */
export async function exchangeCodeForUser(
  provider: AuthProvider,
  code: string,
  origin: string,
  isSandbox: boolean = false
): Promise<AuthUser> {
  const redirectUri = `${origin}/api/auth/callback/${provider}`;

  if (isSandbox) {
    const nameMap: Record<string, string> = {
      github: 'GitHub Engineer (Local Dev)',
      google: 'Google Workspace Architect',
      microsoft: 'Microsoft Cloud Lead',
    };
    const emailDomain = provider === 'google' ? 'google.com' : 'dagr.dev';
    return {
      id: `usr_${provider}_sandbox_${Date.now()}`,
      email: `engineer@${emailDomain}`,
      name: nameMap[provider] || `${provider} User`,
      provider,
      org_id: 'org_dagr_core',
      org_name: 'DAGR Core Engineering',
      created_at: Date.now(),
    };
  }

  if (provider === 'github') {
    return await exchangeGitHubCode(code, redirectUri);
  }

  if (provider === 'google') {
    return await exchangeGoogleCode(code, redirectUri);
  }

  if (provider === 'microsoft') {
    return await exchangeMicrosoftCode(code, redirectUri);
  }

  throw new Error(`Unsupported OAuth provider: ${provider}`);
}

/**
 * Real GitHub OAuth2 token exchange & profile retrieval
 */
async function exchangeGitHubCode(
  code: string,
  redirectUri: string
): Promise<AuthUser> {
  const clientId = process.env.GITHUB_CLIENT_ID!;
  const clientSecret = process.env.GITHUB_CLIENT_SECRET!;

  // 1. Exchange code for access token
  const tokenRes = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify({
      client_id: clientId,
      client_secret: clientSecret,
      code,
      redirect_uri: redirectUri,
    }),
  });

  if (!tokenRes.ok) {
    throw new Error(`GitHub token exchange failed: HTTP ${tokenRes.status}`);
  }

  const tokenData = await tokenRes.json();
  if (tokenData.error || !tokenData.access_token) {
    throw new Error(
      `GitHub OAuth error: ${tokenData.error_description || tokenData.error}`
    );
  }

  const accessToken = tokenData.access_token;

  // 2. Fetch User Profile
  const userRes = await fetch('https://api.github.com/user', {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'User-Agent': 'dagr-hypervisor',
    },
  });

  if (!userRes.ok) {
    throw new Error(`Failed to fetch GitHub profile: HTTP ${userRes.status}`);
  }

  const userData = await userRes.json();

  // 3. Fetch verified primary email if not public in profile
  let primaryEmail = userData.email;
  if (!primaryEmail) {
    const emailsRes = await fetch('https://api.github.com/user/emails', {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'User-Agent': 'dagr-hypervisor',
      },
    });
    if (emailsRes.ok) {
      const emails = (await emailsRes.json()) as Array<{
        email: string;
        primary: boolean;
        verified: boolean;
      }>;
      const primary = emails.find((e) => e.primary && e.verified) || emails[0];
      if (primary) primaryEmail = primary.email;
    }
  }

  const email = primaryEmail || `${userData.login}@users.noreply.github.com`;
  const domain = email.split('@')[1] || 'github.com';
  const orgName = userData.company || domain.replace(/\.[^.]+$/, '');

  return {
    id: `github:${userData.id}`,
    email,
    name: userData.name || userData.login,
    avatar_url: userData.avatar_url,
    provider: 'github',
    org_id: `org_${orgName.toLowerCase().replace(/[^a-z0-9]/g, '_')}`,
    org_name: orgName,
    created_at: Date.now(),
  };
}

/**
 * Real Google OAuth2 token exchange & profile retrieval
 */
async function exchangeGoogleCode(
  code: string,
  redirectUri: string
): Promise<AuthUser> {
  const clientId = process.env.GOOGLE_CLIENT_ID!;
  const clientSecret = process.env.GOOGLE_CLIENT_SECRET!;

  // 1. Exchange code for tokens
  const tokenRes = await fetch('https://oauth2.googleapis.com/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      code,
      client_id: clientId,
      client_secret: clientSecret,
      redirect_uri: redirectUri,
      grant_type: 'authorization_code',
    }),
  });

  if (!tokenRes.ok) {
    throw new Error(`Google token exchange failed: HTTP ${tokenRes.status}`);
  }

  const tokenData = await tokenRes.json();
  if (tokenData.error || !tokenData.access_token) {
    throw new Error(
      `Google OAuth error: ${tokenData.error_description || tokenData.error}`
    );
  }

  // 2. Fetch User Profile
  const userRes = await fetch('https://www.googleapis.com/oauth2/v3/userinfo', {
    headers: { Authorization: `Bearer ${tokenData.access_token}` },
  });

  if (!userRes.ok) {
    throw new Error(`Failed to fetch Google profile: HTTP ${userRes.status}`);
  }

  const userData = await userRes.json();
  const email = userData.email;
  const domain = email.split('@')[1] || 'google.com';
  const orgName =
    userData.hd ||
    (domain.toLowerCase() === 'gmail.com' ? 'Personal' : domain.replace(/\.[^.]+$/, ''));

  return {
    id: `google:${userData.sub}`,
    email,
    name: userData.name || email.split('@')[0],
    avatar_url: userData.picture,
    provider: 'google',
    org_id: `org_${orgName.toLowerCase().replace(/[^a-z0-9]/g, '_')}`,
    org_name: orgName,
    created_at: Date.now(),
  };
}

/**
 * Real Microsoft Entra ID OAuth2 token exchange & profile retrieval
 */
async function exchangeMicrosoftCode(
  code: string,
  redirectUri: string
): Promise<AuthUser> {
  const clientId = process.env.MICROSOFT_CLIENT_ID!;
  const clientSecret = process.env.MICROSOFT_CLIENT_SECRET!;
  const tenant = process.env.MICROSOFT_TENANT_ID || 'common';

  // 1. Exchange code for token
  const tokenRes = await fetch(
    `https://login.microsoftonline.com/${tenant}/oauth2/v2.0/token`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        client_id: clientId,
        client_secret: clientSecret,
        code,
        redirect_uri: redirectUri,
        grant_type: 'authorization_code',
      }),
    }
  );

  if (!tokenRes.ok) {
    throw new Error(`Microsoft token exchange failed: HTTP ${tokenRes.status}`);
  }

  const tokenData = await tokenRes.json();
  if (tokenData.error || !tokenData.access_token) {
    throw new Error(
      `Microsoft OAuth error: ${tokenData.error_description || tokenData.error}`
    );
  }

  // 2. Fetch Graph user profile
  const userRes = await fetch('https://graph.microsoft.com/v1.0/me', {
    headers: { Authorization: `Bearer ${tokenData.access_token}` },
  });

  if (!userRes.ok) {
    throw new Error(`Failed to fetch Microsoft profile: HTTP ${userRes.status}`);
  }

  const userData = await userRes.json();
  const email = userData.mail || userData.userPrincipalName;
  const domain = email.split('@')[1] || 'microsoft.com';
  const orgName = domain.replace(/\.[^.]+$/, '');

  return {
    id: `microsoft:${userData.id}`,
    email,
    name: userData.displayName || email.split('@')[0],
    provider: 'microsoft',
    org_id: `org_${orgName.toLowerCase().replace(/[^a-z0-9]/g, '_')}`,
    org_name: orgName,
    created_at: Date.now(),
  };
}
