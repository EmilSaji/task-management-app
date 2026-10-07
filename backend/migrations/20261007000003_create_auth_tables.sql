-- Two-factor login challenges.
-- The one-time code is never stored: only HMAC-SHA256(OTP_SECRET, challenge_id || code).
CREATE TABLE login_challenges (
    id          UUID PRIMARY KEY,
    user_id     UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    code_hash   TEXT        NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NULL,
    attempts    INTEGER     NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_login_challenges_user_id ON login_challenges (user_id);

-- Audit record of every email the system "sent".
-- Deliberately metadata only: the verification code is NOT persisted here.
CREATE TABLE email_logs (
    id           UUID PRIMARY KEY,
    recipient    TEXT        NOT NULL,
    subject      TEXT        NOT NULL,
    kind         TEXT        NOT NULL,
    challenge_id UUID        NULL REFERENCES login_challenges (id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_email_logs_recipient_created_at ON email_logs (recipient, created_at DESC);
