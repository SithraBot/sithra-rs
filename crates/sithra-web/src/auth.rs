use std::sync::{LazyLock, RwLock};

use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| String::from("sithra"));
    Keys::new(secret.as_bytes())
});

pub static CREDENTIALS: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| {
    let path = std::env::var("CREDENTIALS_PATH").unwrap_or_else(|_| String::from("./credentials"));
    let path = std::path::Path::new(&path);
    if path.exists() {
        let res = std::fs::read_to_string(path).expect("Failed to read credentials file");
        return RwLock::new(Some(res));
    }
    RwLock::new(None)
});

pub async fn flush_credentials() -> Result<(), std::io::Error> {
    let cred = CREDENTIALS.read().unwrap().clone();
    match cred {
        Some(cred) => {
            tokio::fs::write("./credentials", cred).await?;
        }
        None => {
            tokio::fs::remove_file("./credentials").await?;
        }
    }
    Ok(())
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}
impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn auth_verify(token: &Claims) -> Result<(), AuthError> {
    if token
        .hex
        .eq(CREDENTIALS.read().unwrap().as_ref().ok_or(AuthError::CredentialsNotFound)?)
    {
        Ok(())
    } else {
        Err(AuthError::WrongCredentials)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub hex: String,
    #[serde(default)]
    pub exp: usize,
}

pub struct Auth;
impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        log::debug!("auth: {bearer:?}");
        let mut validation = Validation::default();
        validation.validate_exp = false;
        let token_data =
            decode::<Claims>(bearer.token(), &KEYS.decoding, &validation).map_err(|e| {
                log::debug!("auth_error: {e}");
                AuthError::InvalidToken
            })?;
        auth_verify(&token_data.claims)?;
        Ok(Self)
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("Missing credentials")]
    MissingCredentials,
    #[error("Token creation failed")]
    TokenCreation,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Credentials not found")]
    CredentialsNotFound,
    #[error("Credentials already exists")]
    CredentialsAlreadyExists,
    #[error("Write error")]
    FileWriteError(#[from] std::io::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::WrongCredentials => StatusCode::UNAUTHORIZED,
            Self::MissingCredentials | Self::CredentialsAlreadyExists | Self::InvalidToken => {
                StatusCode::BAD_REQUEST
            }
            Self::TokenCreation | Self::CredentialsNotFound | Self::FileWriteError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, self.to_string()).into_response()
    }
}
