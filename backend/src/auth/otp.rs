//! One-time verification codes for email 2FA.
//!
//! Codes are 6 random digits. Only a keyed hash
//! `HMAC-SHA256(OTP_SECRET, challenge_id || ":" || code)` is persisted, so a
//! database leak does not reveal codes, and binding the hash to the challenge
//! id means a hash cannot be replayed against another challenge.

use hmac::{Hmac, Mac};
use rand::{rngs::OsRng, Rng};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_code() -> String {
    format!("{:06}", OsRng.gen_range(0..1_000_000u32))
}

fn mac(secret: &str, challenge_id: Uuid, code: &str) -> HmacSha256 {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(challenge_id.as_bytes());
    mac.update(b":");
    mac.update(code.trim().as_bytes());
    mac
}

pub fn hash_code(secret: &str, challenge_id: Uuid, code: &str) -> String {
    hex::encode(mac(secret, challenge_id, code).finalize().into_bytes())
}

/// Constant-time comparison of a submitted code against the stored hash.
pub fn verify_code(secret: &str, challenge_id: Uuid, code: &str, stored_hash: &str) -> bool {
    let Ok(expected) = hex::decode(stored_hash) else {
        return false;
    };
    mac(secret, challenge_id, code)
        .verify_slice(&expected)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "otp-test-secret-otp-test-secret-123456";

    #[test]
    fn generated_codes_are_six_digits() {
        for _ in 0..100 {
            let code = generate_code();
            assert_eq!(code.len(), 6);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn hash_is_not_the_code_and_verifies() {
        let id = Uuid::new_v4();
        let hash = hash_code(SECRET, id, "123456");
        assert!(!hash.contains("123456"));
        assert!(verify_code(SECRET, id, "123456", &hash));
        assert!(
            verify_code(SECRET, id, " 123456 ", &hash),
            "whitespace is trimmed"
        );
    }

    #[test]
    fn wrong_code_or_other_challenge_fails() {
        let id = Uuid::new_v4();
        let hash = hash_code(SECRET, id, "123456");
        assert!(!verify_code(SECRET, id, "654321", &hash));
        assert!(!verify_code(SECRET, Uuid::new_v4(), "123456", &hash));
        assert!(!verify_code("other-secret", id, "123456", &hash));
        assert!(!verify_code(SECRET, id, "123456", "not-hex"));
    }
}
