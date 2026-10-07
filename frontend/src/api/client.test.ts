import { describe, expect, it, vi } from 'vitest';
import { ApiError, apiRequest, configureAuth } from './client';

function mockFetch(status: number, body: unknown) {
  return vi.spyOn(globalThis, 'fetch').mockResolvedValue(
    new Response(JSON.stringify(body), {
      status,
      headers: { 'Content-Type': 'application/json' },
    }),
  );
}

describe('apiRequest', () => {
  it('attaches the JWT as a Bearer token', async () => {
    configureAuth({ getToken: () => 'jwt-123', onUnauthorized: () => {} });
    const fetchSpy = mockFetch(200, { ok: true });

    await apiRequest('/tasks/view-my-tasks');

    const [url, init] = fetchSpy.mock.calls[0];
    expect(url).toBe('/api/tasks/view-my-tasks');
    expect((init?.headers as Record<string, string>).Authorization).toBe('Bearer jwt-123');
  });

  it('does not attach the token for public endpoints', async () => {
    configureAuth({ getToken: () => 'jwt-123', onUnauthorized: () => {} });
    const fetchSpy = mockFetch(200, {});

    await apiRequest('/auth/login', { method: 'POST', body: {}, auth: false });

    const headers = fetchSpy.mock.calls[0][1]?.headers as Record<string, string>;
    expect(headers.Authorization).toBeUndefined();
  });

  it('maps a 403 to an ApiError with the server message', async () => {
    configureAuth({ getToken: () => 'jwt-bond', onUnauthorized: () => {} });
    mockFetch(403, { error: { code: 'forbidden', message: 'Only admins can perform this action' } });

    const error = await apiRequest('/tasks', { method: 'POST', body: { title: 'x' } }).catch(
      (e: unknown) => e,
    );

    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).isForbidden).toBe(true);
    expect((error as ApiError).code).toBe('forbidden');
    expect((error as ApiError).message).toBe('Only admins can perform this action');
  });

  it('signs the user out when an authenticated request gets 401', async () => {
    const onUnauthorized = vi.fn();
    configureAuth({ getToken: () => 'expired', onUnauthorized });
    mockFetch(401, { error: { code: 'unauthorized', message: 'Invalid or expired access token' } });

    await expect(apiRequest('/tasks/view-my-tasks')).rejects.toBeInstanceOf(ApiError);
    expect(onUnauthorized).toHaveBeenCalledOnce();
  });

  it('reports network failures clearly', async () => {
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new TypeError('Failed to fetch'));
    await expect(apiRequest('/health', { auth: false })).rejects.toMatchObject({
      status: 0,
      code: 'network_error',
    });
  });
});
