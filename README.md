# copier-rustlang-web-app-oidc

Copier template for an axum web app with minijinja templates and oidc auth.

## Developing the template

Unlike most copier templates, `template/` is a **runnable cargo project**.
There is no `{{ project_name }}` placeholder anywhere in it: every file
already contains the default answer, `web-app-oidc`. That means you can
type-check and test the template without rendering it first:

```bash
cd template
cargo check
cargo run -- serve --oauth-provider-url https://your-oidc-provider/
```

IDEs and rust-analyzer also work directly on `template/`.

When the template is rendered (`copier copy`), a post-render task renames
the package in `Cargo.toml`, `Cargo.lock`, `src/main.rs` (APP_NAME) and
`Dockerfile` from the default to the answered `project_name`, keeping the
generated project consistent.

## Using the template

```bash
copier copy https://github.com/andlaz/copier-rustlang-web-app-oidc my-app
cd my-app
```

See `template/README.md` (rendered into the new project) for building and
running instructions.

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
