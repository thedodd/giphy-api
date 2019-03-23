use serde_derive::{Serialize, Deserialize};
use wither_derive::Model;
use mongodb::{
    self, doc, bson,
    coll::options::{
        IndexModel,
    },
    oid::ObjectId,
};

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

/// A user of the system.
#[derive(Model, Serialize, Deserialize, Debug, Clone)]
#[model(collection_name="users")]
pub struct User {
    /// The object's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's email address.
    #[model(index(index="asc", name="unique-email", background="true", unique="true"))]
    #[model(index(index="asc", background="true", with(field="pwhash", index="asc")))]
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
