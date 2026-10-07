import { apiRequest } from './client';
import type { LoginResponse, TokenResponse, User } from './types';

export const authApi = {
  login: (email: string, password: string) =>
    apiRequest<LoginResponse>('/auth/login', {
      method: 'POST',
      body: { email, password },
      auth: false,
    }),

  verifyTwoFactor: (loginChallengeId: string, code: string) =>
    apiRequest<TokenResponse>('/auth/verify-2fa', {
      method: 'POST',
      body: { login_challenge_id: loginChallengeId, code },
      auth: false,
    }),

  me: () => apiRequest<User>('/auth/me'),
};
