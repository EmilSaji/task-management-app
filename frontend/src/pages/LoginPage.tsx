import { useState } from 'react';
import { Navigate, useNavigate } from 'react-router-dom';
import type { LoginResponse, TokenResponse } from '../api/types';
import { LoginForm } from '../components/LoginForm';
import { TwoFactorForm } from '../components/TwoFactorForm';
import { homePathFor, useAuth } from '../context/AuthContext';

type Step = { kind: 'credentials' } | { kind: 'verify'; email: string; challenge: LoginResponse };

export function LoginPage() {
  const { user, signIn } = useAuth();
  const navigate = useNavigate();
  const [step, setStep] = useState<Step>({ kind: 'credentials' });

  if (user) return <Navigate to={homePathFor(user)} replace />;

  const onVerified = (response: TokenResponse) => {
    signIn(response);
    navigate(homePathFor(response.user), { replace: true });
  };

  return (
    <div className="login-page">
      <h1 className="brand-large">Task Manager</h1>
      {step.kind === 'credentials' ? (
        <LoginForm onChallenge={(email, challenge) => setStep({ kind: 'verify', email, challenge })} />
      ) : (
        <TwoFactorForm
          email={step.email}
          challenge={step.challenge}
          onVerified={onVerified}
          onBack={() => setStep({ kind: 'credentials' })}
        />
      )}
    </div>
  );
}
