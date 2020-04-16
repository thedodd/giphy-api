Bonus: Ownership, Borrowing & Lifetimes
=======================================

### Data Ownership
Data is always owned. References are a way to lease out access to the owned data, and lifetimes help you (and the compiler) to keep track of this lease.

```rust
/// The application state object.
#[derive(Clone)]
pub struct State {
    pub db: PgPool,
    pub client: Client,
    pub config: Arc<Config>,
}
```

Let's have a look at our API code and how we lease out access to our config data (see `server/src/api.rs`).

### Embedding a Lifetime
Remember that with references, you are dealing with data that is owned by something else.

```rust
struct DBInfo<'a> {
    name: &'a str,
    tables: u64,
}

fn build_info<'a>(name: &'a str, db: &mut PgConn) -> DBInfo<'a> {
    // Do some work, get some info.
    let tables = get_table_count(db);
    DBInfo{name, tables}
}

fn main() {
    let my_db_name = String::from("oxidize");

    // Build our info struct.
    let info = build_info(&my_db_name, get_db());

    // Report our info.
    metrics.report_info(info); // <-- the lifetime 'a is still alive here.

    // ... do more cool stuff.
}
```

Why is this significant?

- Your code doesn't have to check to see if `info.name` is nil/null/void ... because that doesn't exist in Rust.
- For as long as `'a` is alive and well, that reference to `my_db_name` stands. Can not be mutated. Can not be destroyed.
- No garbage collector needed.

Remember, lifetime rules apply to `&` references. Not to the various pointer types in Rust (Box, Rc, Arc etc), though you could still pass around references to them if needed.
