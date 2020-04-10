// impl Handler<FindUserSavedGifs> for MongoExecutor {
//     type Result = <FindUserSavedGifs as Message>::Result;

//     /// Handle fetching a user's saved GIFs.
//     fn handle(&mut self, msg: FindUserSavedGifs, _: &mut Self::Context) -> Self::Result {
//         // Fetch all saved user GIFs.
//         match SavedGif::find(self.0.clone(), Some(doc!{"user": msg.0}), None) {
//             Ok(gifs) => Ok(gifs),
//             Err(err) => {
//                 error!("Error fatching user's saved GIFs. {:?}", err);
//                 Err(Error::new_ise())
//             }
//         }
//     }
// }

// impl Handler<SaveGif> for MongoExecutor {
//     type Result = <SaveGif as Message>::Result;

//     /// Handle saving a user's GIF.
//     fn handle(&mut self, msg: SaveGif, _: &mut Self::Context) -> Self::Result {
//         let mut model = SavedGif::from((msg.0, msg.1));
//         match model.save(self.0.clone(), None) {
//             Ok(_) => (),
//             Err(err) => {
//                 error!("Error saving user's GIF. {:?}", err);
//                 return Err(Error::new_ise());
//             }
//         }
//         Ok(model)
//     }
// }

// impl Handler<CategorizeGif> for MongoExecutor {
//     type Result = <CategorizeGif as Message>::Result;

//     /// Handle saving a user's GIF.
//     fn handle(&mut self, msg: CategorizeGif, _: &mut Self::Context) -> Self::Result {
//         let (user, gif) = (msg.0, msg.1);
//         let filter = doc!{"user": user, "giphy_id": gif.id};
//         let update = match gif.category.len() > 0 {
//             true => doc!{"$set": doc!{"category": gif.category}},
//             false => doc!{"$set": doc!{"category": Bson::Null}},
//         };
//         let mut options = FindOneAndUpdateOptions::new();
//         options.return_document = Some(ReturnDocument::After);
//         SavedGif::find_one_and_update(self.0.clone(), filter, update, Some(options))
//             .map_err(|mongoerr| {
//                 error!("Error while attempting to find and update GIF category. {:?}", mongoerr);
//                 Error::new_ise()
//             })
//             .and_then(|opt| match opt {
//                 Some(model) => Ok(model),
//                 None => Err(Error::new("Could not find target GIF saved by user.", 400, None)),
//             })
//     }
// }
