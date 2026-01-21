this is a basic tokio/tower/axum/minijinja web app template with
axum-oidc authentication

## Roadmap

- [x] README with full build and run instructions
- [x] basic build ( musl builder, distroless runner ) in a Dockerfile
- [ ] jetbrains run configs
- [ ] integration test w/ embedded keycloak, [see example](https://github.com/pfzetto/axum-oidc/blob/0.6.0/examples/basic/tests/keycloak.rs)
- [ ] compose with keycloak for local debugging/dev
- [x] templates bundled in build.rs
- [ ] dagger golang sdk ci
- [x] clap command and config args
- [x] rustls wrapped listening socket
- [x] a few demo routes including a `/logout`
- [x] axum-template/minijinja set up
