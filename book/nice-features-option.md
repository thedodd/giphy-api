Nice Features: Option
=====================
Instead of `nil`, `None`, or `Null`, in Rust we have the `Option` enum type.

```rust
enum Option<T> {
    Some(T),
    None,
}
```

An example of how we are using an `Option` type in our app code.

```rust
/// A GIF from the Giphy API which has been saved by a user.
struct SavedGif {
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
```

How might we use this?

```rust
// Take the inner value, or a default.
gif.category.unwrap_or_default();

// Take the inner value, or an explicit alternative.
gif.category.unwrap_or(String::from("Woo!"));
gif.category.unwrap_or_else(|| String::from("Woo!")); // Using a closure.

// Match on the structure of the option itself.
// This matches against the possible variants of the type.
match gif.category {
    Some(val) => val,
    None => String::from("New Val"),
}

// If we just want to check for Some(..) or None.
if let Some(val) = gif.category {
    // Use the inner value here.
}
if let None = gif.category {
    // No category, so do something else.
}
```

Why is this great? No more nil pointer dereferencing.

What about our own custom enum types? Here is one that we use heavily in the app.

```rust
/// An API response.
enum Response<D> {
    /// A success payload with data.
    Data(D),
    /// An error payload with an error.
    Error(Error),
}
```
