use std::collections::HashMap;

use actix::prelude::*;
use bcrypt::{BcryptError, DEFAULT_COST, hash, verify};
use common::{
    Error, GiphyGif, CategorizeGifRequest, LoginRequest, RegisterRequest,
};
use log::{error};
use validator::Validate;
use wither::{
    prelude::*,
    mongodb::{
        doc, bson,
        Bson, Error as MongoError, Client, ThreadedClient,
        db::Database, oid::ObjectId,
        coll::options::{FindOneAndUpdateOptions, ReturnDocument},
    }
};

use crate::{
    config::Config,
    models::{SavedGif, User},
};

const INVALID_CREDS: &str = "Invalid credentials provided.";

//////////////////////////////////////////////////////////////////////////////////////////////////
// CreateUser ////////////////////////////////////////////////////////////////////////////////////

/// A message type for creating users.
pub struct CreateUser(pub RegisterRequest);

impl Message for CreateUser {
    type Result = Result<User, Error>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// FindUserSavedGifs /////////////////////////////////////////////////////////////////////////////

/// A message type for looking up a user's saved GIFs.
pub struct FindUserSavedGifs(pub ObjectId);

impl Message for FindUserSavedGifs {
    type Result = Result<Vec<SavedGif>, Error>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// FindUserWithCreds /////////////////////////////////////////////////////////////////////////////

/// A message type for looking up a user with matching creds.
pub struct FindUserWithCreds(pub LoginRequest);

impl Message for FindUserWithCreds {
    type Result = Result<User, Error>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// SaveGif ///////////////////////////////////////////////////////////////////////////////////////

/// A message type for saving a GIF for a user.
pub struct SaveGif(pub ObjectId, pub GiphyGif);

impl Message for SaveGif {
    type Result = Result<SavedGif, Error>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// CategorizeGif /////////////////////////////////////////////////////////////////////////////////

/// A message type for categorizing a GIF.
pub struct CategorizeGif(pub ObjectId, pub CategorizeGifRequest);

impl Message for CategorizeGif {
    type Result = Result<SavedGif, Error>;
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
    type Result = <CreateUser as Message>::Result;

    /// Handle creation of new users.
    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        // Validate given input.
        // FUTURE: ensure password is complex enough.
        let _ = (&msg.0).validate().map_err(|err| {
            let mut fields = HashMap::new();
            err.field_errors().into_iter().for_each(|(k, v)| {
                fields.insert(k.to_string(), v[0].to_string());
            });
            Error::new("Invalid input.", 400, Some(fields))
        })?;

        // Hash the user's password.
        let pwhash = hash(&msg.0.password, DEFAULT_COST).map_err(|err| {
            error!("Failed to hash given password. {:?}", err);
            Error::new_ise()
        })?;

        // Build new model instance and attempt to insert into DB.
        // If email is already in use, this will return a unique index violation.
        let email = msg.0.email.to_lowercase(); // Make things consistent.
        let mut user = User::new(email, pwhash);
        let _ = match user.save(self.0.clone(), None) {
            Ok(()) => Ok(()),
            Err(MongoError::OperationError(ref val)) if val.starts_with("E11000") => {
                Err(Error::new("Given email is already in use.", 400, None))
            }
            Err(err) => {
                error!("Error saving user model. {:?}", err);
                Err(Error::new_ise())
            }
        }?;

        Ok(user)
    }
}

impl Handler<FindUserSavedGifs> for MongoExecutor {
    type Result = <FindUserSavedGifs as Message>::Result;

    /// Handle fetching a user's saved GIFs.
    fn handle(&mut self, msg: FindUserSavedGifs, _: &mut Self::Context) -> Self::Result {
        // Fetch all saved user GIFs.
        match SavedGif::find(self.0.clone(), Some(doc!{"user": msg.0}), None) {
            Ok(gifs) => Ok(gifs),
            Err(err) => {
                error!("Error fatching user's saved GIFs. {:?}", err);
                Err(Error::new_ise())
            }
        }
    }
}

impl Handler<FindUserWithCreds> for MongoExecutor {
    type Result = <FindUserWithCreds as Message>::Result;

    /// Handle lookup of user with matching creds.
    fn handle(&mut self, msg: FindUserWithCreds, _: &mut Self::Context) -> Self::Result {
        // Search for user with given credentials.
        let email = msg.0.email.to_lowercase(); // Make things consistent.
        let user = match User::find_one(self.0.clone(), Some(doc!{"email": email}), None) {
            Ok(Some(user)) => Ok(user),
            Ok(None) => {
                Err(Error::new(INVALID_CREDS, 400, None))
            }
            Err(err) => {
                error!("Error while looking up user. {:?}", err);
                Err(Error::new_ise())
            }
        }?;

        // Check the user's creds.
        match verify(&msg.0.password, &user.pwhash) {
            Ok(is_valid) => match is_valid {
                true => Ok(user),
                false => Err(Error::new(INVALID_CREDS, 400, None))
            }
            Err(BcryptError::InvalidPassword) => {
                Err(Error::new(INVALID_CREDS, 400, None))
            }
            Err(err) => {
                error!("Error from bcrypt while checking user's password. {:?}", err);
                Err(Error::new_ise())
            }
        }
    }
}

impl Handler<SaveGif> for MongoExecutor {
    type Result = <SaveGif as Message>::Result;

    /// Handle saving a user's GIF.
    fn handle(&mut self, msg: SaveGif, _: &mut Self::Context) -> Self::Result {
        let mut model = SavedGif::from((msg.0, msg.1));
        match model.save(self.0.clone(), None) {
            Ok(_) => (),
            Err(err) => {
                error!("Error saving user's GIF. {:?}", err);
                return Err(Error::new_ise());
            }
        }
        Ok(model)
    }
}

impl Handler<CategorizeGif> for MongoExecutor {
    type Result = <CategorizeGif as Message>::Result;

    /// Handle saving a user's GIF.
    fn handle(&mut self, msg: CategorizeGif, _: &mut Self::Context) -> Self::Result {
        let (user, gif) = (msg.0, msg.1);
        let filter = doc!{"user": user, "giphy_id": gif.id};
        let update = match gif.category.len() > 0 {
            true => doc!{"$set": doc!{"category": gif.category}},
            false => doc!{"$set": doc!{"category": Bson::Null}},
        };
        let mut options = FindOneAndUpdateOptions::new();
        options.return_document = Some(ReturnDocument::After);
        SavedGif::find_one_and_update(self.0.clone(), filter, update, Some(options))
            .map_err(|mongoerr| {
                error!("Error while attempting to find and update GIF category. {:?}", mongoerr);
                Error::new_ise()
            })
            .and_then(|opt| match opt {
                Some(model) => Ok(model),
                None => Err(Error::new("Could not find target GIF saved by user.", 400, None)),
            })
    }
}
