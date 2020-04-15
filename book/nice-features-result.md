Nice Features: Result
=====================
Instead of `except Exception as ex` or `if err != nil` or `rescue ExceptionType` or `try .. catch` or (the worst) `if ret < 0`, in Rust we have another enum type: `Result`.

```rust
/// An enum, generic over a success type and an error type.
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Ok, so how do we use a result? Here is an example from our app (slightly abridged).

```rust
/// Our error struct.
struct Error {/* ... */}

fn from_jwt(/* ... */) -> Result<Claims, Error>
    // ... snip ...
    let claims = // Decode a token into our `Claims` structure.
    claims.must_not_be_expired()?;
    // ... snip ...
}
```

That `claims.must_not_be_expired()` call returns a `Result`. If an error comes up, Rust has a dedicated syntax element — the postfix `?` operator — to perform an "early return" when a `Result::Err(..)` is encountered.

What's more? Rust performs automatic type coercion with the `?` operator. What does this mean?

```rust
// The std From trait.
trait From<T> {
    fn from(T) -> Self;
}
```

How is this used?

```rust
// Let's say `must_not_be_expired()` returns a different error type:
fn must_not_be_expired(&self) -> Result<(), ExpirationError> {/*...*/}

impl From<ExpirationError> for Error {
    fn from(src: ExpirationError) -> Error {
        Error{
            // Use the values from src here.
        }
    }
}
```

So, when we suffix `claims.must_not_be_expired()` with a `?`, in this context Rust will automatically use the `From` impl we have above to convert the type for us.

Using Rust's `match` syntax for structural matching also works as expected with results.

```rust
match my_result {
    Ok(data) => data,
    Err(err) => {
        // Trace the error ... log the error ... transform the error ... whatever.
        tracing::error!("{}", err);
        return Err(err);
    }
}
```

Lots of methods are available on the `Result` type as well.

```rust
Result::Ok(0)
    .map(|val| val + 1)
    // Let's change the error type if one is encountered.
    .map_err(|_err| "something bad happened!")
    .and_then(|val| {
        if val > 0 {
            Ok(val)
        } else {
            Err("not good!!!")
        }
    })
```

The Rust standard syntax prelude includes the discriminants of the Result type for direct use, as seen above. So you can directly use `Ok(..)` & `Err(..)` in your code as a shorthand. Rust will infer the appropriate types based on function signatures and the like.
