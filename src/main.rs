#[macro_use]
extern crate juniper;
#[macro_use]
extern crate serde_derive;

mod error;
mod graphql_api;
mod healthz;
mod metrics;
mod orgchart;
mod proxy;
mod search;
mod session;
mod settings;

use crate::graphql_api::app::graphql_app;
use crate::healthz::healthz_app;
use crate::metrics::metrics_app;
use crate::orgchart::app::orgchart_app;
use crate::search::app::search_app;
use crate::session::app::session_app;

use actix_web::middleware::Logger;
use actix_web::web::{self, Data};
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use dino_park_gate::provider::Provider;
use dino_park_gate::scope::ScopeAndUserAuth;
use log::info;
use std::io::Error;
use std::io::ErrorKind;

fn map_io_err(e: impl Into<failure::Error>) -> Error {
    Error::new(ErrorKind::Other, e.into())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    ::std::env::set_var(
        "RUST_LOG",
        "actix_web=info,dino_park_fence=info,dino_park_gate=info",
    );
    env_logger::init();
    info!("building the fence");
    let m = metrics::Metrics::new().map_err(map_io_err)?;
    let s = settings::Settings::new().map_err(map_io_err)?;
    let cis_client = CisClient::from_settings(&s.cis).await.map_err(map_io_err)?;
    let dino_park_settings = s.dino_park;
    let provider = Provider::from_issuer(&s.auth).await.map_err(map_io_err)?;
    // Start http server
    HttpServer::new(move || {
        let scope_middleware = ScopeAndUserAuth::new(provider.clone()).public();
        App::new()
            .wrap(Logger::default().exclude("/healthz"))
            .app_data(Data::new(m.clone()))
            .service(
                web::scope("/api/v4/")
                    .wrap(scope_middleware)
                    .service(graphql_app(cis_client.clone(), &dino_park_settings))
                    .service(search_app(&dino_park_settings.search))
                    .service(orgchart_app(&dino_park_settings.orgchart)),
            )
            .service(session_app())
            .service(healthz_app())
            .service(metrics_app())
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
