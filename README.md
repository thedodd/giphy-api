giphy api
=========
A WebAssembly application that allows a user to search for and save animated GIFs to a user profile using the [GIPHY API](https://developers.giphy.com/docs/).

### overview
#### api
The API is structured as a very simple old school REST API written in Rust. The client & server use the same exact data models for communicating over the network. All interaction is protected by JWT authN/authZ.

#### client app
The client app is a WebAssembly (WASM) application built using Rust.

#### database
We are using MongoDB as the backend for this system. We are using an ODM, called [Wither](https://github.com/thedodd/wither), which I created. This allows us to deal with our MongoDB collections in a model-first fashion.

#### setup
First, ensure you have [docker](https://docs.docker.com/install/#supported-platforms) & [docker compose](https://docs.docker.com/compose/install/) installed on your system. Everything in this system is intended to run entirely within docker.

```bash
# Once you have docker in place, booting the entire system is just one comamnd.
# NOTE: when first run, it will take some time to compile the client & server.
docker-compose up -d

# Stream the logs to ensure everything has come online as needed.
docker-compose logs -f

# You can access the MongoDB instance via the following command.
docker-compose exec mongo mongo
```

Now you're ready to start using the app. Simply navigate to http://localhost:8080 to get started.

----

### deep dive
##### auth
We are using `2048` bit RSA asymmetric keys for creating and verifying our JWTs. The code block below shows how to create a new key pair. The keys must be base64 encoded before being passed into the container runtime environment.
```bash
# Generate new private & public key pair.
openssl genrsa -out /tmp/keypair.pem 2048

# Extract the private key.
openssl rsa -in /tmp/keypair.pem -out /tmp/private.key

# Extract the public key.
openssl rsa -in /tmp/keypair.pem -pubout -out /tmp/public.key
```

##### development
For rapid development, start with the standard docker compose setup described above. Next, we will bring down the server and then run a new copy which will volume mount this repo's `static` directory, and we will have it run in watch mode so that the server will recompile anytime the server code changes.

```bash
# Bring down any running copy of the server.
docker-compose rm -sfv server

# Bring up a new copy which mounts ./static & runs in watch mode.
docker-compose run -v ./static:/api/static -p 8080:8080 server cargo make watch-server-run
```

A few things to note:
- the server will now be running in watch mode & will respond to any changes which take place in the server code.
- the `static` directory will be mounted by the server, so any changes to the files there will be served by the server.

From here you can use `cargo make watch-client` to watch the client code and run its pipeline when its code changes.

----

### demo images mobile
<p>
    <img height="300px" src=".demo-images/0-login.png"/>
    <img height="300px" src=".demo-images/1-search.png"/>
    <img height="300px" src=".demo-images/2-navbar.png"/>
    <img height="300px" src=".demo-images/3-search-results.png"/>
    <img height="300px" src=".demo-images/4-favorites-and-categorization.png"/>
</p>
