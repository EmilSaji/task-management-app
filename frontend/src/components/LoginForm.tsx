import { useState } from 'react';
import type { FormEvent } from 'react';
import { authApi } from '../api/auth';
import { errorMessage } from '../api/client';
import type { LoginResponse } from '../api/types';
import { Alert } from './Alert';

interface Props {
  onChallenge: (email: string, challenge: LoginResponse) => void;
}

const DEMO_ACCOUNTS = [
  { label: 'Admin', email: 'admin@example.com', password: 'Admin@123' },
  { label: 'James Bond', email: 'jamesbond@example.com', password: 'Bond@007' },
];

/** Step 1: email + password. On success the API returns a 2FA challenge (no token). */
export function LoginForm({ onChallenge }: Props) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      const challenge = await authApi.login(email.trim(), password);
      onChallenge(email.trim(), challenge);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form className="card form" onSubmit={submit} noValidate>
      <h2>Sign in</h2>
      {error && <Alert variant="error">{error}</Alert>}
      <label>
        Email
        <input
          type="email"
          autoComplete="username"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />
      </label>
      <label>
        Password
        <input
          type="password"
          autoComplete="current-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
      </label>
      <button className="btn btn-primary" type="submit" disabled={submitting || !email || !password}>
        {submitting ? 'Sending code…' : 'Continue'}
      </button>

      {import.meta.env.DEV && (
        <div className="demo-accounts">
          <span className="muted">Demo accounts (run POST /seed/users first):</span>
          {DEMO_ACCOUNTS.map((a) => (
            <button
              key={a.email}
              type="button"
              className="btn btn-ghost btn-small"
              onClick={() => {
                setEmail(a.email);
                setPassword(a.password);
              }}
            >
              {a.label}
            </button>
          ))}
        </div>
      )}
    </form>
  );
}
