use actix::prelude::*;
use bcrypt::{BcryptError, DEFAULT_COST, hash, verify};
use log::{error};
use wither::{
    prelude::*,
    mongodb::{
        doc, bson,
        Error as MongoError, Client, ThreadedClient, db::Database,
    }
};
use validator::{Validate};
use validator_derive::Validate;

use crate::{
    config::Config,
    proto::api::Error,
    models::User,
};

//////////////////////////////////////////////////////////////////////////////////////////////////
// CreateUser ////////////////////////////////////////////////////////////////////////////////////

/// A message type for creating users.
#[derive(Validate)]
pub struct CreateUser {
    #[validate(email(message="Must provide a valid email address."))]
    pub email: String,
    #[validate(length(min="6", message="Password must be at least 6 characters in length."))]
    pub password: String,
}

impl Message for CreateUser {
    type Result = Result<User, Error>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// CreateUser ////////////////////////////////////////////////////////////////////////////////////

/// A message type for looking up a user with matching creds.
pub struct FindUserWithCreds {
    pub email: String,
    pub password: String,
}

impl Message for FindUserWithCreds {
    type Result = Result<User, Error>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// MongoExecutor /////////////////////////////////////////////////////////////////////////////////

/// An actor for interfacing with MongoDB.
#[derive(Clone)]
pub struct MongoExecutor(pub Database);

impl Actor for MongoExecutor {
    type Context = SyncContext<Self>;
}

impl MongoExecutor {
    /// Create a new instance.
    pub fn new(cfg: &Config) -> Result<Self, MongoError> {
        let client = Client::with_uri(&cfg.backend_connection_string)?;
        let db = client.db(&cfg.backend_database);
        Ok(Self(db))
    }
}

impl Handler<CreateUser> for MongoExecutor {
    type Result = Result<User, Error>;

    /// Handle creation of new users.
    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        // Validate given input.
        // FUTURE: ensure password is complex enough.
        let _ = msg.validate().map_err(|err| {
            Error::new(&err.to_string(), 400, None, None)
        })?;

        // Hash the user's password.
        let pwhash = hash(&msg.password, DEFAULT_COST).map_err(|err| {
            error!("Failed to hash given password. {:?}", err);
            Error::new_ise(None, None)
        })?;

        // Build new model instance and attempt to insert into DB.
        // If email is already in use, this will return a unique index violation.
        let email = msg.email.to_lowercase(); // Make things consistent.
        let mut user = User::new(email, pwhash);
        let _ = match user.save(self.0.clone(), None) {
            Ok(()) => Ok(()),
            Err(MongoError::OperationError(ref val)) if val.starts_with("E11000") => {
                Err(Error::new("Given email is already in use.", 400, None, None))
            }
            Err(err) => {
                error!("Error saving user model. {:?}", err);
                Err(Error::new_ise(None, None))
            }
        }?;

        Ok(user)
    }
}

impl Handler<FindUserWithCreds> for MongoExecutor {
    type Result = Result<User, Error>;

    /// Handle lookup of user with matching creds.
    fn handle(&mut self, msg: FindUserWithCreds, _: &mut Self::Context) -> Self::Result {
        // Search for user with given credentials.
        let email = msg.email.to_lowercase(); // Make things consistent.
        let user = match User::find_one(self.0.clone(), Some(doc!{"email": email}), None) {
            Ok(Some(user)) => Ok(user),
            Ok(None) => {
                Err(Error::new("Invalid credentials provided.", 400, None, None))
            }
            Err(err) => {
                error!("Error while looking up user. {:?}", err);
                Err(Error::new_ise(None, None))
            }
        }?;

        // Check the user's creds.
        match verify(&msg.password, &user.pwhash) {
            Ok(_) => Ok(user),
            Err(BcryptError::InvalidPassword) => {
                Err(Error::new("Invalid credentials provided.", 400, None, None))
            }
            Err(err) => {
                error!("Error from bcrypt while checking user's password. {:?}", err);
                Err(Error::new_ise(None, None))
            }
        }
    }
}
