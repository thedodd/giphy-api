use common::GiphyGif;
use serde_derive::{Serialize, Deserialize};
use wither_derive::Model;
use mongodb::{
    self, doc, bson,
    coll::options::{
        IndexModel,
    },
    oid::ObjectId,
};

//////////////////////////////////////////////////////////////////////////////////////////////////
// FavoriteGif ///////////////////////////////////////////////////////////////////////////////////

/// A GIF from the Giphy API which has been saved by a user.
#[derive(Model, Serialize, Deserialize, Clone)]
#[model(collection_name="saved_gifs")]
pub struct SavedGif {
    /// The object's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The ID of the user which has saved this GIF.
    #[model(index(index="dsc", background="true", unique="true", with(field="giphy_id", index="dsc")))]
    pub user: ObjectId,

    /// The ID of this GIF in the Giphy system.
    pub giphy_id: String,

    /// The title of the GIF.
    pub title: String,

    /// The URL of the GIF.
    pub url: String,

    /// The category given to this GIF by the user.
    pub category: Option<String>,
}

impl From<(ObjectId, GiphyGif)> for SavedGif {
    /// Perform the conversion.
    fn from((user, gif): (ObjectId, GiphyGif)) -> Self {
        Self{id: None, user, giphy_id: gif.id, title: gif.title, url: gif.url, category: None}
    }
}

impl From<SavedGif> for GiphyGif {
    /// Perform the conversion.
    fn from(gif: SavedGif) -> Self {
        Self{id: gif.giphy_id, title: gif.title, url: gif.url, is_saved: true, category: gif.category}
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////////////////////////

/// A user of the system.
#[derive(Model, Serialize, Deserialize, Clone)]
#[model(collection_name="users")]
pub struct User {
    /// The object's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's email address.
    #[model(index(index="dsc", name="unique-email", background="true", unique="true"))]
    #[model(index(index="dsc", background="true", with(field="pwhash", index="dsc")))]
    pub email: String,

    /// The user's password hash.
    pub pwhash: String,
}

impl User {
    /// Create a new instance.
    pub fn new(email: String, pwhash: String) -> Self {
        User{id: None, email, pwhash}
    }
}
