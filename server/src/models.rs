use std::collections::HashMap;

use futures::prelude::*;
use futures::stream::{TryStream, TryStreamExt};

use common::Error;
use crate::{Tx, PgPoolConn};

//////////////////////////////////////////////////////////////////////////////////////////////////
// FavoriteGif ///////////////////////////////////////////////////////////////////////////////////

/// A GIF from the Giphy API which has been saved by a user.
#[derive(Clone, sqlx::FromRow)]
pub struct SavedGif {
    /// Object ID.
    pub id: i64,
    /// The ID of the user which has saved this GIF.
    pub user: i64,
    /// The ID of this GIF in the Giphy system.
    pub giphy_id: String,
    /// The title of the GIF.
    pub title: String,
    /// The URL of the GIF.
    pub url: String,
    /// The category given to this GIF by the user.
    pub category: Option<String>,
}

impl SavedGif {
    /// Find all gifs saved by the specified user.
    pub async fn for_user_matching_ids<'a>(user: i64, ids: &'a [String], db: &'a mut PgPoolConn) -> Result<HashMap<String, SavedGif>, Error> {
        let stream = sqlx::query_as!(SavedGif, r#"SELECT * FROM public.saved_gifs WHERE "user"=$1 AND giphy_id=ANY($2);"#, user, ids)
            .fetch(db);
        Ok(stream
            .try_fold(HashMap::new(), |mut acc, gif| async move {
                acc.insert(gif.giphy_id.clone(), gif);
                Ok(acc)
            })
            .map_err(Error::from)
            .await?)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////////////////////////

/// A user of the system.
#[derive(Clone)]
pub struct User {
    /// Object ID.
    pub id: i64,
    /// The user's email address.
    pub email: String,
    /// The user's password hash.
    pub pwhash: String,
}

impl User {
    /// Insert a new record.
    pub async fn insert(email: String, pwhash: String, tx: &mut Tx) -> Result<Self, Error> {
        Ok(sqlx::query_as!(User, "INSERT INTO public.users (email, pwhash) VALUES ($1, $2) RETURNING *;", email, pwhash)
            .fetch_one(tx)
            .await
            .map_err(|err| match err {
                sqlx::Error::Database(dberr) => {
                    match dberr.constraint_name() {
                        Some(constraint) if constraint == "users_email_key" => {
                            Error::new("That email address is already taken.", 400, None)
                        }
                        _ => Error::from(sqlx::Error::Database(dberr)), // Just resurface the error.
                    }
                }
                _ => Error::new_ise(),
            })?)
    }

    /// Find a user record by the given email.
    pub async fn find_by_email(email: String, db: &mut PgPoolConn) -> Result<Option<Self>, Error> {
        Ok(sqlx::query_as!(User, "SELECT * FROM public.users WHERE email=$1;", email)
            .fetch_optional(db)
            .await
            .map_err(Error::from)?)
    }

    /// Find a user record by the given id.
    pub async fn find_by_id(id: i64, db: &mut PgPoolConn) -> Result<Option<Self>, Error> {
        Ok(sqlx::query_as!(User, "SELECT * FROM public.users WHERE id=$1;", id)
            .fetch_optional(db)
            .await
            .map_err(Error::from)?)
    }

    /// Transform into the common::User model.
    pub fn into_common(self, jwt: String) -> common::User {
        common::User{id: self.id, email: self.email, jwt}
    }
}
