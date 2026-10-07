import type { ApiErrorBody } from './types';

const BASE_URL = (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? '/api';

/** Error thrown for any non-2xx API response (or network failure, status 0). */
export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }

  get isForbidden() {
    return this.status === 403;
  }

  get isUnauthorized() {
    return this.status === 401;
  }
}

// The auth layer registers how to read the current token and what to do on
// an expired session; UI components never handle headers themselves.
let getToken: () => string | null = () => null;
let onUnauthorized: () => void = () => {};

export function configureAuth(options: {
  getToken: () => string | null;
  onUnauthorized: () => void;
}) {
  getToken = options.getToken;
  onUnauthorized = options.onUnauthorized;
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PATCH';
  body?: unknown;
  /** Attach the JWT (default true). */
  auth?: boolean;
}

export async function apiRequest<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = 'GET', body, auth = true } = options;
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (body !== undefined) headers['Content-Type'] = 'application/json';

  const token = auth ? getToken() : null;
  if (token) headers.Authorization = `Bearer ${token}`;

  let response: Response;
  try {
    response = await fetch(`${BASE_URL}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    throw new ApiError(0, 'network_error', 'Cannot reach the API. Is the backend running?');
  }

  const text = await response.text();
  const data: unknown = text ? safeJson(text) : null;

  if (!response.ok) {
    const err = (data as ApiErrorBody | null)?.error;
    const apiError = new ApiError(
      response.status,
      err?.code ?? 'http_error',
      err?.message ?? `Request failed with status ${response.status}`,
    );
    // An authenticated request rejected with 401 means the session is gone.
    if (apiError.isUnauthorized && token) onUnauthorized();
    throw apiError;
  }
  return data as T;
}

function safeJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

/** Human-friendly message for any thrown value. */
export function errorMessage(error: unknown): string {
  if (error instanceof ApiError) return error.message;
  if (error instanceof Error) return error.message;
  return 'Something went wrong.';
}
