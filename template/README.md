simple axum webapp with oidc

## Roadmap

- [x] i ran `copier copy`
- [ ] Loading...

## Building

> needs: docker, gcr.io and docker.io access

```bash
docker build -t web-app-oidc . 
```

## Running

### Running locally in docker, with a remote oauth provider

> needs: docker, oauth client registration at provider

for testing purposes, create a self-signed key pair below for
your domain ( `$CN` ) and alt domains ( `$ALTNAME` ).
Make sure either is or contains your test domain ( here `web-app-oidc` ) or `localhost`. 

Notice the `chgrp` of sensitive files to the non-root group 
in the distroless base image `65532`. The sudo in here is for updating group
ownership to a group that `$UID` is not member of

```bash
CN="web-app-oidc.k8s-auto-1.us-west-1.aws.andlaz.io"; \
ALTNAME="DNS:web-app-oidc"; \
openssl req -x509 -newkey rsa:4096 -sha256 -days 365 -nodes \
      -keyout target/tls.key -out target/tls.crt \
      -subj /C=IN/ST=MH/L=PUN/O=TW/OU=IT/CN="${CN}" \
      -extensions ext \
      -config <(cat <<EOF
[ext]
subjectAltName=$ALTNAME
EOF
) && \
sudo chgrp 65532 target/tls.key && \
chmod 0640 target/tls.key

```

next, prepare the oauth credentials
```bash
if [ ! -f "target/.env" ]; then
    mkdir -p target && \
    touch target/.env && \
    # this is the uid/gid in distroless/static-debian12:nonroot
    sudo chgrp 65532 target/.env && \
    chmod 0640 target/.env && \
(cat <<EOF
OAUTH_PROVIDER_CLIENT_ID=...
OAUTH_PROVIDER_CLIENT_SECRET=...
EOF
) > target/.env
fi
```

finally, start the service, bind-mounting the above
and passing in the non-sensitive parts of the oauth
provider configuration

```bash
OAUTH_PROVIDER_URL=https://dev-5klruudiueqy8bx3.us.auth0.com/; \
docker run -ti --rm \
  --name web-app-oidc \
  --publish 127.0.0.1:8443:8443 \
  --mount type=bind,source=$PWD/target/.env,target=/.env \
  --mount type=bind,source=$PWD/target/tls.key,target=/tls.key \
  --mount type=bind,source=$PWD/target/tls.crt,target=/tls.crt \
  web-app-oidc \
  serve \
  --listen 0.0.0.0:8443 \
  --redirect-url https://web-app-oidc:8443/ \
  --tls-key /tls.key \
  --tls-cert /tls.crt \
  --oauth-provider-url $OAUTH_PROVIDER_URL
```
