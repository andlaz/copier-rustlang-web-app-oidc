FROM clux/muslrust:stable AS builder

RUN mkdir /app
WORKDIR /app
COPY template ./template
COPY Cargo.* ./
COPY build.rs ./
COPY src ./src

RUN cargo build --release

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /app/target/**/release/web-app-oidc /

EXPOSE 8080/tcp
EXPOSE 8443/tcp

ENTRYPOINT ["/web-app-oidc"]
