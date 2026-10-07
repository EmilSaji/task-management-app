//! Outgoing email. Real delivery is out of scope for this assignment, so the
//! [`ConsoleMailer`] prints the email to the server log and keeps it in an
//! in-memory [`DevOutbox`] that the dev-only endpoint reads from.
//!
//! The plaintext code therefore only ever lives in process memory / stdout,
//! never in the database. Swapping in SMTP means adding another `Mailer` impl.

use std::{collections::VecDeque, sync::Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{dto::dev::DevEmail, error::AppResult};

const OUTBOX_CAPACITY: usize = 100;

#[derive(Debug, Clone)]
pub struct VerificationEmail {
    pub to: String,
    pub code: String,
    pub challenge_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

impl VerificationEmail {
    pub const SUBJECT: &'static str = "Your Task Manager verification code";

    pub fn body(&self) -> String {
        format!(
            "Your verification code is {}. It expires at {} and can only be used once.",
            self.code,
            self.expires_at.format("%H:%M:%S UTC")
        )
    }
}

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verification_code(&self, email: &VerificationEmail) -> AppResult<()>;
}

/// Bounded in-memory list of recently sent emails (development only).
#[derive(Default)]
pub struct DevOutbox {
    emails: Mutex<VecDeque<DevEmail>>,
}

impl DevOutbox {
    pub fn push(&self, email: DevEmail) {
        let mut emails = self.emails.lock().expect("outbox lock poisoned");
        if emails.len() == OUTBOX_CAPACITY {
            emails.pop_front();
        }
        emails.push_back(email);
    }

    /// Most recent email, optionally filtered by recipient (case-insensitive).
    pub fn latest(&self, recipient: Option<&str>) -> Option<DevEmail> {
        let emails = self.emails.lock().expect("outbox lock poisoned");
        emails
            .iter()
            .rev()
            .find(|e| recipient.is_none_or(|r| e.to.eq_ignore_ascii_case(r.trim())))
            .cloned()
    }

    pub fn clear(&self) {
        self.emails.lock().expect("outbox lock poisoned").clear();
    }
}

pub struct ConsoleMailer {
    outbox: std::sync::Arc<DevOutbox>,
}

impl ConsoleMailer {
    pub fn new(outbox: std::sync::Arc<DevOutbox>) -> Self {
        Self { outbox }
    }
}

#[async_trait]
impl Mailer for ConsoleMailer {
    async fn send_verification_code(&self, email: &VerificationEmail) -> AppResult<()> {
        tracing::info!(
            to = %email.to,
            challenge_id = %email.challenge_id,
            "📧 [dev email] {} — code: {}",
            VerificationEmail::SUBJECT,
            email.code
        );
        self.outbox.push(DevEmail {
            to: email.to.clone(),
            subject: VerificationEmail::SUBJECT.to_string(),
            body: email.body(),
            code: email.code.clone(),
            login_challenge_id: email.challenge_id,
            sent_at: Utc::now(),
        });
        Ok(())
    }
}
