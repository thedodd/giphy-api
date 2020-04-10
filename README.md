giphy api
=========
A WebAssembly application that allows a user to search for and save animated GIFs to a user profile using the [GIPHY API](https://developers.giphy.com/docs/).

### overview
The API is structured as a very simple JSON/RPC API built using [actix.rs](https://actix.rs/). The client & server use the same exact data models for communicating over the network. All interaction is protected by JWT authN/authZ.

The client app is a WebAssembly (WASM) application built using Rust.

We are using Postgres for data storage & [launchbadge/sqlx](https://github.com/launchbadge/sqlx) for the interface.

#### setup
First, you'll need Rust. Head on over to https://rustup.rs/ and follow the instructions there to setup the Rust toolchain. As we are building our Rust code to target the WASM instruction set, and now that we have `rustup` on our machine, let's add the needed compiler target:

```bash
# Add the WASM 32-bit instruction set as a compilation target.
rustup target add wasm32-unknown-unknown
# While we're at it, let's install the wasm-bindgen-cli
# which we will need for our WASM builds later.
cargo install wasm-bindgen-cli --version=0.2.55
```

Second, you'll need to have docker in place to run the Postgres database, check out the [docker installation docs](https://docs.docker.com/get-docker/) if you don't already have docker on your machine.

Now that you have all of the tools in place, let's bring up the DB and build our Rust code.
```bash
# Boot Postgres. This will also initialize our tables.
docker run -d --name postgres \
    -e POSTGRES_PASSWORD=pgpass \
    -v `pwd`/pg.sql:/docker-entrypoint-initdb.d/pg.sql \
    postgres
# Build the UI.
build -p client --release --target wasm32-unknown-unknown
# Run wasm-bindgen on our output WASM.
wasm-bindgen target/wasm32-unknown-unknown/release/client.wasm --no-modules --out-dir ./static
# Now, we run our API which will also serve our WASM bundle, HTML and other assets.
cargo run -p server --release
```
Now you're ready to start using the app. Simply navigate to http://localhost:9000 to get started.

----

### demo images mobile
<p>
    <img height="300px" src=".demo-images/0-login.png"/>
    <img height="300px" src=".demo-images/1-search.png"/>
    <img height="300px" src=".demo-images/2-navbar.png"/>
    <img height="300px" src=".demo-images/3-search-results.png"/>
    <img height="300px" src=".demo-images/4-favorites-and-categorization.png"/>
</p>
