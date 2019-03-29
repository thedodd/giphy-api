giphy api
=========
Design and implement a web application that allows a user to search for and save animated GIFs to a user profile using the [GIPHY API](https://developers.giphy.com/docs/).

**Requirements:**
- [x] Provide users with the ability to register and login to your application.
- [x] As well as the ability to search for animated GIFs from the GIPHY API.
- [x] As well as the ability to save and view their favorite GIFs to their profile.
- [ ] Also provide the user with the ability to categorize these saved GIFs (ex: funny, animals, etc.).
- [x] User data should be stored in a database.
- [x] Basic application security practices should be implemented (OWASP Top 9).
- [x] GIFs available on your application should be limited to a G-Rating.

**NB:** items marked as `// FUTURE:` are simply items which I've skipped over in the name of brevity, but which we would normally want to finish implementing before deploying.

### overview
#### api
The API is implemented as a [Protocol Buffers](https://developers.google.com/protocol-buffers/docs/overview) based Websockets API.

All interaction with this API is performed over Websockets and must adhere to the protocol outlined in the API's main protobuf file.

### getting started
We are using MongoDB as the backend for this system. First, ensure you have [docker](https://docs.docker.com/install/#supported-platforms) & [docker compose](https://docs.docker.com/compose/install/) installed on your system.

```bash
# Next, from the root of this repository, boot up the MongoDB instance.
docker-compose up -d mongo

# You can check its logs to ensure everything came up properly.
docker-compose logs -f mongo

# You can access the MongoDB instance via the following command.
docker-compose exec mongo mongo
```










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
