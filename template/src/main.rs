use anyhow::{Context, anyhow};
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::extract::FromRef;
use axum::http::Uri;
use axum::response::IntoResponse;
use axum::routing::get;
use axum_oidc::error::MiddlewareError;
use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer, OidcLoginLayer, OidcRpInitiatedLogout};
use axum_template::engine::Engine;
use clap::{Parser, Subcommand};
use minijinja::Environment;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use tower::ServiceBuilder;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing::info;

mod handling;
mod telemetry;
mod cli;

static APP_NAME: &str = "web-app-oidc";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    pub(crate) command: cli::Commands,
}

type AppEngine = Engine<Environment<'static>>;

#[derive(Clone, FromRef)]
struct AppState {
    engine: AppEngine,
    redirect_url: String,
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    // Logging
    let subscriber =
        telemetry::get_subscriber(APP_NAME.into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Search for a .env
    match dotenvy::dotenv() {
        Ok(p) => info!("Loaded .env from {:?}", p),
        Err(e) => info!("No .env file loaded: {}", e),
    }

    // install a built-in crypto provider
    tracing::info_span!("crypto_provider_load").in_scope(|| -> anyhow::Result<_, _> {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|e| anyhow!("Failed to initialize crypto provider: {:?}", e))
    })?;

    // parse CLI args and execute command
    let cli = Cli::try_parse()?;
    match cli.command {
        cli::Commands::Serve {
            listen,
            redirect_url,
            tls_cert,
            tls_key,
            oauth_provider_url,
            oauth_provider_client_id,
            oauth_provider_client_secret,
        } => {
            // set up ctrl-c/signal handlers
            let handle = axum_server::Handle::new();
            let _signal = tokio::spawn(shutdown_signal(handle.clone()));

            // configure oidc layers
            let oidc_login_service = ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
                    e.into_response()
                }))
                .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

            let oidc_auth_service = ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
                    e.into_response()
                }))
                .layer(
                    OidcAuthLayer::<EmptyAdditionalClaims>::discover_client(
                        Uri::from_maybe_shared(redirect_url.clone()).context("Failed to parse redirect url")?,
                        oauth_provider_url,
                        oauth_provider_client_id,
                        oauth_provider_client_secret,
                        vec![],
                    )
                        .await.context("Failed to create OIDC client via provider discovery endpoint `.well-known/openid-configuration`")?,
                );

            // set up jinja
            let mut jinja = Environment::new();
            // /build.rs places these
            minijinja_embed::load_templates!(&mut jinja);

            let app = Router::new()
                .route("/logout", get(handling::logout))
                .route("/task/sum", get(handling::task::sum))
                .layer(oidc_login_service)
                .layer(oidc_auth_service)
                .route("/", get(handling::greet))
                .route("/health", get(|| async { "OK" }))
                .with_state(AppState {
                    engine: Engine::from(jinja),
                    redirect_url: redirect_url.clone(),
                })
                .layer(session_layer(time::Duration::seconds(120)));

            info!("🚀 Listening on {}, CTRL-C to exit", listen);
            if let (Some(tls_cert), Some(tls_key)) = (tls_cert, tls_key) {
                // start TLS on --listen
                let tls_config =
                    axum_server::tls_rustls::RustlsConfig::from_pem_file(tls_cert, tls_key)
                        .await
                        .context("Failed to read TLS key-pair")?;

                axum_server::bind_rustls(listen.clone(), tls_config)
                    .handle(handle)
                    .serve(app.into_make_service())
                    .await
                    .context("Axum has exited with an error")?;
            } else {
                // start non-TLS on --listen
                axum_server::bind(listen.clone())
                    .handle(handle)
                    .serve(app.into_make_service())
                    .await
                    .context("Axum has exited with an error")?;
            };

            anyhow::Ok(())
        }
    }
}

// Wait for ctr-c or a unix signal
// and trigger graceful shutdown of axum
async fn shutdown_signal(handle: axum_server::Handle<SocketAddr>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
}

fn session_layer(session_timeout: time::Duration) -> SessionManagerLayer<MemoryStore> {
    let session_store = MemoryStore::default();
    SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(session_timeout))
}
