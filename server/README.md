server
======

We are using `2048` bit RSA asymmetric keys for creating and verifying our JWTs. The code block below shows how to create a new key pair. The keys must be base64 encoded before being passed into the container runtime environment.
```bash
# Generate new private & public key pair.
openssl genrsa -out /tmp/keypair.pem 2048

# Extract the private key.
openssl rsa -in /tmp/keypair.pem -out /tmp/private.key

# Extract the public key.
openssl rsa -in /tmp/keypair.pem -pubout -out /tmp/public.key
```
