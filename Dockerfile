FROM rust:1.33-slim

LABEL maintainer="Anthony Josiah Dodd <adodd@docql.io>"
EXPOSE 8080
WORKDIR /api

# Add a few system deps.
RUN apt-get update && apt-get install -y make pkg-config libssl-dev && \
    rustup target add wasm32-unknown-unknown && \
    cargo install cargo-make --version 0.17.0 && \
    cargo install cargo-watch --version 7.2.0

COPY ./client client
COPY ./common common
COPY ./server server
COPY ./static static
COPY ./Makefile.toml Makefile.toml
COPY ./Cargo.toml Cargo.toml
COPY ./Cargo.lock Cargo.lock

# This will build the client & server in release mode.
RUN cargo make app-build
CMD ["/api/target/release/server"]
