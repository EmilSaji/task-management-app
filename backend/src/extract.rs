use axum::{
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors};

use crate::error::AppError;

/// `Json<T>` that also runs `validator` rules and returns our JSON error
/// envelope (400 for malformed JSON, 422 for validation failures).
pub struct ValidJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| AppError::BadRequest(rejection.body_text()))?;
        value
            .validate()
            .map_err(|errors| AppError::Validation(format_errors(&errors)))?;
        Ok(Self(value))
    }
}

fn format_errors(errors: &ValidationErrors) -> String {
    let mut messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |e| {
                let msg = e
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| e.code.to_string());
                format!("{field}: {msg}")
            })
        })
        .collect();
    messages.sort();
    messages.join("; ")
}
