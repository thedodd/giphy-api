use common::Error;

use crate::{Tx, PgPoolConn};

//////////////////////////////////////////////////////////////////////////////////////////////////
// FavoriteGif ///////////////////////////////////////////////////////////////////////////////////

/// A GIF from the Giphy API which has been saved by a user.
#[derive(Clone)]
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

impl From<SavedGif> for common::GiphyGif {
    fn from(src: SavedGif) -> Self {
        Self{
            id: src.id,
            title: src.title,
            url: src.url,
            is_saved: true,
            category: src.category,
        }
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

    /// Find a user record by the given email, the pwhash MUST be checked in order to confirm authentication.
    pub async fn find_by_email(email: String, db: &mut PgPoolConn) -> Result<Option<Self>, Error> {
        Ok(sqlx::query_as!(User, "SELECT * FROM public.users WHERE email=$1;", email)
            .fetch_optional(db)
            .await
            .map_err(Error::from)?)
    }

    /// Transform into the common::User model.
    pub fn into_common(self, jwt: String) -> common::User {
        common::User{id: self.id, email: self.email, jwt}
    }
}
