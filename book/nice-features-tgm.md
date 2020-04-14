Nice Features: Traits, Generics & MetaProgramming
=================================================

### Traits
Some languages have Interfaces, some languages have Protocols, Rust was trying really hard to be the cool kid, so it has Traits instead.

```rust
trait Worker {
    fn do_work(&self) -> Result<(), Error>;
}

struct ThreadedWorker;

impl Worker for ThreadedWorker {
    fn do_work(&self) -> Result<(), Error> {
        Err(Error::default()) // FAIL!
    }
}
```

Trait inheritence is also supported. `trait Worker: Awesome + Cool + Nifty {..}` requires that any implementor of the `Worker` trait must also implement `Awesome`, `Cool`, and `Nifty`.

### Generics
The enums we've studied so far (Option & Result) are both generic types. Here is a generic data type of our own which we use in this app.

```rust
/// An API response.
pub enum Response<D> {
    /// A success payload with data.
    Data(D),
    /// An error payload with an error.
    Error(Error),
}
```

Our response struct carries a data payload within its `Data` variant, and an error in its `Error` variant. At this point, we can use any type for `D`.

Often times we don't actually want to allow any lowsy old type to be used though. How do constraint which types can be used?

```rust
// We can constrain inline.
fn ddos_attack<T: HttpEndpoint>(target: T) {..}

// Or we can constrain with a `where` clause,
// which is nice when you have lots of constraints.
fn ddos_attack<T, C>(target: T, ctx: C)
    where
        T: HttpEndpoint,
        C: Context + Send + Sync,
{..}
```

### MetaProgramming
There are a few kinds in Rust, here is one you will use ALL THE TIME.

```rust
/// An API response.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag="result", content="payload")]
pub enum Response<D> {
    /// A success payload with data.
    #[serde(rename="data")]
    Data(D),
    /// An error payload with an error.
    #[serde(rename="error")]
    Error(Error),
}
```

In this case, the `#[derive(Serialize)]` attribute (focusing only on `Serialize` for now), invokes a function **at compile-type** which will run Rust code over the AST of this enum and generate more code. In this case, in generates code to allow this enum to be serialized into various data formats (JSON, YAML, &c). These are called "procedural macros".

The `#[serde(..)]` attribute is a "helper attribute" of the Serialize & Deserialize macros defined in the serde library code itself, and modifies the macro's behavior.
