use chrono::{Duration, Utc};
use uuid::Uuid;

use super::normalize_email;
use crate::{
    auth::{otp, password},
    dto::auth::{LoginRequest, LoginResponse, TokenResponse, VerifyTwoFactorRequest},
    error::{AppError, AppResult, TwoFactorError},
    mail::VerificationEmail,
    repositories::{challenges, email_logs, users},
    state::AppState,
};

/// Step 1: check credentials, create a 2FA challenge and email the code.
/// Never returns a token.
pub async fn login(state: &AppState, req: LoginRequest) -> AppResult<LoginResponse> {
    let email = normalize_email(&req.email);
    let user = users::find_by_email(&state.db, &email).await?;

    // Always run one Argon2 verification so unknown emails take as long as wrong passwords.
    let hash = user
        .as_ref()
        .map(|u| u.hashed_password.clone())
        .unwrap_or_else(|| password::dummy_hash().to_string());
    let password_ok = password::verify_password_blocking(req.password, hash).await?;

    let user = match user {
        Some(user) if password_ok => user,
        _ => return Err(AppError::Unauthorized("Invalid email or password".into())),
    };

    let challenge_id = Uuid::new_v4();
    let code = otp::generate_code();
    let code_hash = otp::hash_code(&state.config.otp_secret, challenge_id, &code);
    let expires_at = Utc::now() + Duration::seconds(state.config.otp_ttl_seconds);

    let mut tx = state.db.begin().await?;
    challenges::revoke_pending_for_user(&mut *tx, user.id).await?;
    challenges::insert(&mut *tx, challenge_id, user.id, &code_hash, expires_at).await?;
    email_logs::insert(
        &mut *tx,
        &user.email,
        VerificationEmail::SUBJECT,
        "two_factor_code",
        Some(challenge_id),
    )
    .await?;
    tx.commit().await?;

    state
        .mailer
        .send_verification_code(&VerificationEmail {
            to: user.email.clone(),
            code,
            challenge_id,
            expires_at,
        })
        .await?;

    tracing::info!(user_id = %user.id, %challenge_id, "2FA challenge created");

    Ok(LoginResponse {
        login_challenge_id: challenge_id,
        two_factor_required: true,
        expires_at,
        message: format!(
            "A verification code was sent to {}. It expires in {} minutes.",
            mask_email(&user.email),
            state.config.otp_ttl_seconds / 60
        ),
    })
}

/// Step 2: verify the emailed code and, only then, issue a JWT.
pub async fn verify_two_factor(
    state: &AppState,
    req: VerifyTwoFactorRequest,
) -> AppResult<TokenResponse> {
    let challenge = challenges::find(&state.db, req.login_challenge_id)
        .await?
        .ok_or(AppError::TwoFactor(TwoFactorError::UnknownChallenge))?;

    if challenge.consumed_at.is_some() {
        return Err(AppError::TwoFactor(TwoFactorError::AlreadyUsed));
    }
    if challenge.expires_at <= Utc::now() {
        return Err(AppError::TwoFactor(TwoFactorError::Expired));
    }
    if challenge.attempts >= state.config.otp_max_attempts {
        return Err(AppError::TwoFactor(TwoFactorError::TooManyAttempts));
    }

    if !otp::verify_code(
        &state.config.otp_secret,
        challenge.id,
        &req.code,
        &challenge.code_hash,
    ) {
        challenges::increment_attempts(&state.db, challenge.id).await?;
        return Err(AppError::TwoFactor(TwoFactorError::InvalidCode));
    }

    // Single-use guarantee: only one request can flip consumed_at from NULL.
    if !challenges::consume(&state.db, challenge.id).await? {
        return Err(AppError::TwoFactor(TwoFactorError::AlreadyUsed));
    }

    let user = users::find_by_id(&state.db, challenge.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User no longer exists".into()))?;

    let access_token = state.jwt.issue(user.id, &user.email, user.role)?;
    tracing::info!(user_id = %user.id, "2FA verified, token issued");

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: state.jwt.ttl_seconds(),
        user: user.into(),
    })
}

/// `jamesbond@example.com` -> `j*******@example.com`
fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) if !local.is_empty() => {
            let first = &local[..1];
            format!("{first}{}@{domain}", "*".repeat(local.len() - 1))
        }
        _ => email.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::mask_email;

    #[test]
    fn masks_local_part() {
        assert_eq!(mask_email("jamesbond@example.com"), "j********@example.com");
        assert_eq!(mask_email("a@b.c"), "a@b.c");
    }
}
