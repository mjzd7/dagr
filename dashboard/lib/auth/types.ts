export type AuthProvider = 'github' | 'google' | 'microsoft' | 'email' | 'apikey';

export interface AuthUser {
  id: string;
  email: string;
  name: string;
  avatar_url?: string;
  provider: AuthProvider;
  org_id: string;
  org_name: string;
  created_at: number;
}

export interface OAuthStatePayload {
  provider: AuthProvider;
  nonce: string;
  origin: string;
  returnUrl?: string;
  timestamp: number;
}

export interface SessionPayload {
  user: AuthUser;
  iat: number;
  exp: number;
}

export interface EmailVerificationToken {
  email: string;
  code: string;
  exp: number;
}
