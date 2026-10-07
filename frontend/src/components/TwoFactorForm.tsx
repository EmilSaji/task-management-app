import { useState } from 'react';
import type { FormEvent } from 'react';
import { authApi } from '../api/auth';
import { errorMessage } from '../api/client';
import { devApi } from '../api/dev';
import type { LoginResponse, TokenResponse } from '../api/types';
import { Alert } from './Alert';

interface Props {
  email: string;
  challenge: LoginResponse;
  onVerified: (response: TokenResponse) => void;
  onBack: () => void;
}

/** Step 2: enter the emailed 6-digit code to receive the JWT. */
export function TwoFactorForm({ email, challenge, onVerified, onBack }: Props) {
  const [code, setCode] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [devNote, setDevNote] = useState<string | null>(null);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      onVerified(await authApi.verifyTwoFactor(challenge.login_challenge_id, code.trim()));
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  };

  // Development convenience: read the code from GET /dev/email-logs/latest.
  const fetchDevCode = async () => {
    setError(null);
    try {
      const mail = await devApi.latestEmail(email);
      setCode(mail.code);
      setDevNote(`Code read from the dev mailbox (sent to ${mail.to}).`);
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  const expires = new Date(challenge.expires_at).toLocaleTimeString();

  return (
    <form className="card form" onSubmit={submit} noValidate>
      <h2>Two-factor verification</h2>
      <Alert variant="info">
        {challenge.message} <span className="muted">(valid until {expires})</span>
      </Alert>
      {error && <Alert variant="error">{error}</Alert>}
      {devNote && !error && <Alert variant="success">{devNote}</Alert>}
      <label>
        Verification code
        <input
          inputMode="numeric"
          autoComplete="one-time-code"
          pattern="\d{6}"
          maxLength={6}
          placeholder="123456"
          value={code}
          onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))}
          autoFocus
        />
      </label>
      <button className="btn btn-primary" type="submit" disabled={submitting || code.length !== 6}>
        {submitting ? 'Verifying…' : 'Verify and sign in'}
      </button>
      <div className="row">
        <button type="button" className="btn btn-ghost" onClick={onBack}>
          ← Back
        </button>
        {import.meta.env.DEV && (
          <button type="button" className="btn btn-ghost" onClick={fetchDevCode}>
            Fetch code from dev mailbox
          </button>
        )}
      </div>
    </form>
  );
}
