use actix_web::HttpRequest;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::prelude::*;
use common::Error;
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

use crate::models;

const DEFAULT_USER_TOKEN_EXP: i64 = 60 * 60 * 2; // Two hours.

/// Generate a bcrypt hash of the given password string.
pub fn hash_pw(pw: &str) -> Result<String, Error> {
    Ok(hash(pw, DEFAULT_COST).map_err(|err| {
        tracing::error!("Failed to hash given password. {}", err);
        Error::new_ise()
    })?)
}

pub fn verify_user_pw(user: &models::User, pw: &str) -> Result<(), Error> {
    let is_valid = verify(pw, &user.pwhash).map_err(|err| {
        tracing::error!("Failed to hash given password. {}", err);
        Error::new_ise()
    })?;
    if !is_valid {
        return Err(Error::new_invalid_credentials());
    }
    Ok(())
}

/// The definition of our JWT claims structure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// The ID of the entity for which the token was issued.
    pub sub: i64,
    /// The time which the token was issued.
    ///
    /// This is an epoch timestamp. **Seconds** since the epoch (not milliseconds, or nanoseconds).
    pub iat: i64,
    /// The time when this token will expire, if any.
    ///
    /// This is an epoch timestamp. **Seconds** since the epoch (not milliseconds, or nanoseconds).
    pub exp: i64,
}

impl Claims {
    /// Generate a new JWT for the user specified by ID.
    pub fn new_for_user(private_key: &jwt::EncodingKey, sub: i64) -> Result<String, Error> {
        // Build a new claims body.
        let now = Utc::now();
        let exp_duration = chrono::Duration::seconds(DEFAULT_USER_TOKEN_EXP);
        let claims = Claims{sub, iat: now.timestamp(), exp: (now + exp_duration).timestamp()};
        // Generate token.
        Ok(jwt::encode(&jwt::Header::new(jwt::Algorithm::RS512), &claims, private_key).map_err(|err| {
            tracing::error!({error=%err}, "Error generating new JWT for user.");
            Error::new_ise()
        })?)
    }

    /// Extract JWT claims from the given request.
    pub async fn from_request(request: &HttpRequest, pub_key: &jwt::DecodingKey<'static>) -> Result<Self, Error> {
        // Extract auth val from header.
        let authval = match request.headers().get("authorization") {
            Some(val) => match val.to_str() {
                Ok(strval) => strval,
                Err(_) => return Err(Error::new("Invalid contents of header 'Authorization'.".into(), 401, None)),
            }
            // No auth header presented.
            None => return Err(Error::new("No credentials provided in request.".into(), 401, None)),
        };

        // Evaluate token's presented auth scheme.
        let mut token_segments = authval.splitn(2, " ");
        let is_schema_valid = match token_segments.next() {
            Some(scheme) if scheme.to_lowercase() == "bearer" => {
                true
            }
            _ => false,
        };
        if !is_schema_valid {
            return Err(Error::new("Invalid authorization scheme specified, must be 'bearer'.".into(), 401, None));
        }

        // Scheme is good, now ensure we have a token with non-zero size.
        let token = match token_segments.next() {
            Some(token) => token,
            None => return Err(Error::new("Invalid authorization token specified. It appears to be zero-size.".into(), 401, None)),
        };

        // Extract claims.
        Ok(Self::from_jwt(&token, pub_key)?)
    }

    /// Attempt to extract a claims body from the given JWT.
    ///
    /// This routine will check the veracity of the token's signature, ensuring the token
    /// has not been tampered with, and that the token is not expired.
    pub fn from_jwt(jwt: &str, pub_key: &jwt::DecodingKey<'static>) -> Result<Self, Error> {
        // Decode token & extract claims.
        let claims = match jwt::decode::<Self>(jwt, pub_key, &jwt::Validation::new(jwt::Algorithm::RS512)) {
            Ok(t) => t.claims,
            Err(_) => Err(Error::new_invalid_token())?,
        };

        // Ensure the claims are valid.
        claims.must_not_be_expired()?;
        Ok(claims)
    }

    /// Validate that the given claims have not expired.
    fn must_not_be_expired(&self) -> Result<(), Error> {
        let now = Utc::now().timestamp();
        if now > self.exp {
            Err(Error::new_token_expired())
        } else {
            Ok(())
        }
    }
}
