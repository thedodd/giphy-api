use chrono::prelude::*;
use common::Error;
use log::{error};
use serde_derive::{Deserialize, Serialize};
use simple_jwt::{Algorithm, encode, decode};
use time::Duration;

/// The definition of our JWT claims structure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// The ID of the entity for which the token was issued.
    pub sub: String,

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
    pub fn new<'a>(private_key: &'a str, sub: String) -> Result<String, Error> {
        // Build a new claims body.
        let now = Utc::now();
        let exp = now + Duration::minutes(30);
        let claims = Claims {sub, iat: now.timestamp(), exp: exp.timestamp()};

        // Generate token.
        match encode(&claims, private_key, Algorithm::RS256) {
            Ok(t) => Ok(t),
            Err(err) => {
                error!("Error generating new JWT for user: {}", err);
                Err(Error::new_ise())
            }
        }
    }

    /// Attempt to extract a claims body from the given JWT.
    ///
    /// This routine will check the veracity of the token's signature, ensuring the token
    /// has not been tampered with — which also ensures it was issued by our system — and
    /// will also ensure that the token is not expired.
    pub fn from_jwt<'a>(jwt: &'a str, pub_key: &'a str) -> Result<Claims, Error> {
        // Decode token & extract claims.
        let claims = match decode::<Claims>(jwt, pub_key) {
            Ok(t) => t,
            Err(err) => {
                error!("Error decoding JWT. {}", err);
                return Err(Error::new("Unauthorized. Invalid credentials provided.", 401, None));
            },
        };

        // Ensure the claims are valid.
        match claims.must_not_be_expired() {
            Ok(_) => Ok(claims),
            Err(err) => Err(err),
        }
    }

    /// Validate that the given claims have not expired.
    ///
    /// This routine is private, as it only needs to be called when the `Claims` object
    /// is initially extracted from its JWT.
    fn must_not_be_expired(&self) -> Result<(), Error> {
        if Utc::now().timestamp() > self.exp {
            Err(Error::new("Unauthorized. Given credentials have expired.", 401, None))
        } else {
            Ok(())
        }
    }
}
