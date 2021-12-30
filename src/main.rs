#[macro_use]
extern crate tracing;

#[macro_use]
extern crate lazy_static;


mod users;
mod db;
mod auth;
mod notifications;
mod utils;
mod rooms;
mod playlists;

use std::sync::Arc;
use std::time::Duration;
use poem::{Endpoint, EndpointExt, IntoResponse, Request, Response, Result, Route, Server};
use poem::listener::TcpListener;
use poem::middleware::Cors;
use poem::http::Method;
use poem_openapi::{OpenApiService, Tags};

use concread::arcache::{ARCache, ARCacheBuilder};
use tokio::time::Instant;

#[derive(Tags)]
pub enum ApiTags {
    User,
    Auth,
    Notifications,
    Rooms,
    Playlists,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info,poem=debug,scylla=info");
    }
    tracing_subscriber::fmt::init();

    let session = db::connect("127.0.0.1:9042").await?;
    let cache: ARCache<String, String> = ARCacheBuilder::new()
        .set_size(1024, 10)
        .build()
        .unwrap();

    let api_service = OpenApiService::new(
        (
            users::UsersApi,
            auth::AuthApi,
            notifications::NotificationsApi,
            rooms::RoomsApi,
            playlists::PlaylistsApi,
        ),
        "Spooderfy API",
        "1.0.0"
        )
        .description("The Spooderfy api system.")
        .server("http://127.0.0.1:8000/api/v0");

    let ui = api_service.redoc();
    let spec = api_service.spec();

    let app = Route::new()
        .nest("/api/v0", api_service)
        .nest("/ui", ui)
        .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        .with(
            Cors::new()
                .allow_origin("127.0.0.1:3000")
                .allow_header("*")
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        )
        .around(log)
        .data(session)
        .data(Arc::new(cache));

    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run_with_graceful_shutdown(
            app,
            async move {
                let _ = tokio::signal::ctrl_c().await;
            },
            Some(Duration::from_secs(2)),
        )
        .await?;

    Ok(())
}


async fn log<E: Endpoint>(next: E, req: Request) -> Result<Response> {
    let method = req.method().clone();
    let path = req.uri().clone();

    let start = Instant::now();
    let res = next.call(req).await;
    let elapsed = start.elapsed();

    match res {
        Ok(r) => {
            let resp = r.into_response();

            info!(
                "{} -> {} {} [ {:?} ] - {:?}",
                method.as_str(),
                resp.status().as_u16(),
                resp.status().canonical_reason().unwrap_or(""),
                elapsed,
                path.path(),
            );

            Ok(resp)
        },
        Err(e) => {

            let resp = e.as_response();

            if resp.status().as_u16() >= 500 {
                error!("{}", &e);
            }

            info!(
                "{} -> {} {} [ {:?} ] - {:?}",
                method.as_str(),
                resp.status().as_u16(),
                resp.status().canonical_reason().unwrap_or(""),
                elapsed,
                path.path(),
            );

            Err(e)
        }
    }
}
