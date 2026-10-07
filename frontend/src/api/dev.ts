import { apiRequest } from './client';
import type { DevEmail } from './types';

/** Development-only helpers (the backend only mounts these when APP_ENV=development). */
export const devApi = {
  latestEmail: (email: string) =>
    apiRequest<DevEmail>(`/dev/email-logs/latest?email=${encodeURIComponent(email)}`, {
      auth: false,
    }),
};
