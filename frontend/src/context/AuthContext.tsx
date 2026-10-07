import { createContext, useCallback, useContext, useMemo, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { configureAuth } from '../api/client';
import type { TokenResponse, User } from '../api/types';

const STORAGE_KEY = 'taskapp.session';

interface Session {
  token: string;
  user: User;
}

interface AuthContextValue {
  token: string | null;
  user: User | null;
  isAdmin: boolean;
  signIn: (response: TokenResponse) => void;
  signOut: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function loadSession(): Session | null {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as Session) : null;
  } catch {
    return null;
  }
}

function saveSession(session: Session | null) {
  try {
    if (session) sessionStorage.setItem(STORAGE_KEY, JSON.stringify(session));
    else sessionStorage.removeItem(STORAGE_KEY);
  } catch {
    // Storage unavailable (private mode etc.): the session just won't survive a reload.
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<Session | null>(loadSession);

  const signIn = useCallback((response: TokenResponse) => {
    const next = { token: response.access_token, user: response.user };
    saveSession(next);
    setSession(next);
  }, []);

  const signOut = useCallback(() => {
    saveSession(null);
    setSession(null);
  }, []);

  // Let the API client read the current token and end the session on 401.
  // The ref is updated during render and the client is wired up once, before
  // any child effect runs, so the first request after login carries the token.
  const sessionRef = useRef(session);
  sessionRef.current = session;
  useState(() =>
    configureAuth({ getToken: () => sessionRef.current?.token ?? null, onUnauthorized: signOut }),
  );

  const value = useMemo<AuthContextValue>(
    () => ({
      token: session?.token ?? null,
      user: session?.user ?? null,
      isAdmin: session?.user.role === 'admin',
      signIn,
      signOut,
    }),
    [session, signIn, signOut],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside <AuthProvider>');
  return ctx;
}

export function homePathFor(user: User | null): string {
  return user?.role === 'admin' ? '/admin' : '/my-tasks';
}
