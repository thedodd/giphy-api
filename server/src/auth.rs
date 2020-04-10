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
    pub fn new_for_user<'a>(private_key: &'a jwt::EncodingKey, sub: i64) -> Result<String, Error> {
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

    // /// Attempt to extract a claims body from the given JWT.
    // ///
    // /// This routine will check the veracity of the token's signature, ensuring the token
    // /// has not been tampered with — which also ensures it was issued by our system — and
    // /// will also ensure that the token is not expired.
    // pub fn from_jwt<'a>(jwt: &'a str, pub_key: &'a str) -> Result<Claims, Error> {
    //     // Decode token & extract claims.
    //     let claims = match decode::<Claims>(jwt, pub_key) {
    //         Ok(t) => t,
    //         Err(err) => {
    //             error!("Error decoding JWT. {}", err);
    //             return Err(Error::new("Unauthorized. Invalid credentials provided.", 401, None));
    //         },
    //     };

    //     // Ensure the claims are valid.
    //     match claims.must_not_be_expired() {
    //         Ok(_) => Ok(claims),
    //         Err(err) => Err(err),
    //     }
    // }

    // /// Validate that the given claims have not expired.
    // ///
    // /// This routine is private, as it only needs to be called when the `Claims` object
    // /// is initially extracted from its JWT.
    // fn must_not_be_expired(&self) -> Result<(), Error> {
    //     if Utc::now().timestamp() > self.exp {
    //         Err(Error::new("Unauthorized. Given credentials have expired.", 401, None))
    //     } else {
    //         Ok(())
    //     }
    // }
}
