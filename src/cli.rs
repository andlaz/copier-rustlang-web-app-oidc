use std::net::SocketAddr;
use std::path::PathBuf;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Commands {
    Serve {
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        listen: SocketAddr,
        #[arg(
            long,
            help = "The base URL that the OAuth provider will redirect to after login.\
             Usually resolves to public ingress or the default listen address above in testing\
             ( which is the default )",
            default_value = "http://localhost:8080"
        )]
        redirect_url: String,
        #[arg(long)]
        tls_cert: Option<PathBuf>,
        #[arg(long)]
        tls_key: Option<PathBuf>,
        #[arg(long)]
        oauth_provider_url: String,
        #[arg(long, env)]
        oauth_provider_client_id: String,
        #[arg(long, env)]
        oauth_provider_client_secret: Option<String>,
    },
}