use crate::AppEngine;
use axum::response::IntoResponse;
use axum_oidc::{EmptyAdditionalClaims, OidcClaims, OidcRpInitiatedLogout};
use axum_template::RenderHtml;
use serde::Serialize;

pub(crate) mod task {
    use super::*;

    #[derive(Debug, Serialize)]
    struct SumContext {
        name: String,
        task_sum: u32,
    }
    pub(crate) async fn sum(
        claims: OidcClaims<EmptyAdditionalClaims>,
        engine: AppEngine,
    ) -> impl IntoResponse {
        RenderHtml(
            "task/sum.html",
            engine,
            SumContext {
                name: claims.subject().to_string(),
                task_sum: 42,
            },
        )
    }
}

pub(crate) async fn greet(engine: AppEngine) -> impl IntoResponse {
    RenderHtml("index.html", engine, ())
}

// New logout handler: reads an optional id_token from the session (if present),
// then issues the OIDC RP-initiated logout redirect, and attaches a Set-Cookie
// header that expires the local `tower-sessions` cookie so the browser drops it.
// We avoid calling any specific `destroy()` API on the session to be robust to
// different tower-sessions versions; expiring the cookie client-side ensures
// the local session is discarded.

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::Response;
use axum::http::Uri;
use tower_sessions::Session;

#[derive(Clone)]
struct LogoutState {
    // placeholder if you want to pass additional logout-specific config later
}

pub(crate) async fn logout(
    logout: OidcRpInitiatedLogout,
    session: Session,
    State(app_state): State<crate::AppState>,
) -> impl IntoResponse {
    // Try to read id_token if the login flow stored one under this key.
    // This is optional; many providers allow logout without id_token_hint.
    let _id_token = session.get::<String>("id_token");

    // Expire the cookie on the client by adding a Set-Cookie header.
    // The cookie name used by tower-sessions defaults to `tower-sessions`.
    // Match attributes to the session_layer configuration (SameSite=Lax, HttpOnly).
    // For production, ensure `Secure` and appropriate `SameSite` are set.
    let cookie_clear = "tower-sessions=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax";

    // Build the OIDC logout redirect response. Use the redirect_url stored in AppState
    // if available; otherwise fallback to '/'.
    let post_logout_uri = match Uri::from_maybe_shared(app_state.redirect_url.clone()) {
        Ok(u) => u,
        Err(_) => Uri::from_static("/"),
    };

    let logout_resp = logout.with_post_logout_redirect(post_logout_uri);

    // Attach the cookie clear header to the response produced by the OIDC helper.
    let mut response: Response = logout_resp.into_response();
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_static(cookie_clear),
    );

    response
}
