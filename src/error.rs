use crate::crypto::random_zbase32;
use crate::utils::SecureRandom;
use http::StatusCode;
use log::{debug, error, info};
use std::error::Error;
use std::fmt;

/// Union of all possible runtime error types.
#[derive(Debug)]
pub enum BrokerError {
    /// User input error, which results in 400
    Input(String),
    /// User input error with a specific OAuth code, which results in 400.
    SpecificInput {
        error: String,
        error_description: String,
    },
    /// Identity provider error, which results in 503
    Provider(String),
    /// Identity provider request error, which results in 400
    ProviderInput(String),
    /// Internal errors, which result in 500
    Internal(String),
    /// User was rate limited, results in 413
    RateLimited,
    /// User session not found, results in 400
    SessionExpired,
    /// Result status used by bridges to cancel a request
    ProviderCancelled,
}

impl BrokerError {
    /// Log this error at the appropriate log level.
    /// If `rng` is set, internal errors return a reference number for the error.
    pub async fn log(&self, rng: Option<&SecureRandom>) -> Option<String> {
        match self {
            // User errors only at debug level.
            BrokerError::Input(_)
            | BrokerError::SpecificInput { .. }
            | BrokerError::ProviderInput(_)
            | BrokerError::RateLimited
            | BrokerError::SessionExpired
            | BrokerError::ProviderCancelled => {
                debug!("{self}");
                None
            }
            // Provider errors can be noteworthy, especially when
            // the issue is network related.
            BrokerError::Provider(_) => {
                info!("{self}");
                None
            }
            // Internal errors should ring alarm bells.
            BrokerError::Internal(_) => {
                if let Some(rng) = rng {
                    let reference = random_zbase32(6, rng).await;
                    error!("[REF:{reference}] {self}");
                    Some(reference)
                } else {
                    error!("{self}");
                    None
                }
            }
        }
    }

    /// Get the HTTP status code for this error.
    pub fn http_status_code(&self) -> StatusCode {
        match *self {
            BrokerError::Input(_)
            | BrokerError::SpecificInput { .. }
            | BrokerError::SessionExpired
            | BrokerError::ProviderInput(_) => StatusCode::BAD_REQUEST,
            BrokerError::Provider(_) => StatusCode::SERVICE_UNAVAILABLE,
            BrokerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BrokerError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            // Internal status that should never bubble this far
            BrokerError::ProviderCancelled => unreachable!(),
        }
    }

    /// Get the OAuth2 error code for this error
    pub fn oauth_error_code(&self) -> &str {
        match *self {
            BrokerError::Input(_) | BrokerError::SessionExpired => "invalid_request",
            BrokerError::SpecificInput { ref error, .. } => error,
            BrokerError::Provider(_) | BrokerError::ProviderInput(_) => "temporarily_unavailable",
            BrokerError::Internal(_) => "server_error",
            BrokerError::RateLimited => "access_denied",
            // Internal status that should never bubble this far
            BrokerError::ProviderCancelled => unreachable!(),
        }
    }
}

impl Error for BrokerError {}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            BrokerError::Input(ref description)
            | BrokerError::Provider(ref description)
            | BrokerError::ProviderInput(ref description)
            | BrokerError::Internal(ref description) => description,
            BrokerError::SpecificInput {
                ref error_description,
                ..
            } => error_description,
            BrokerError::RateLimited => "too many requests",
            BrokerError::SessionExpired => "session has expired",
            BrokerError::ProviderCancelled => "bridge cancelled the request",
        })
    }
}

/// Result type with `BrokerError` for errors.
pub type BrokerResult<T> = Result<T, BrokerError>;
