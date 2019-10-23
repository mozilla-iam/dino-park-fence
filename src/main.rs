#[macro_use]
extern crate juniper;
#[macro_use]
extern crate serde_derive;

mod graphql_api;
mod healthz;
mod orgchart;
mod proxy;
mod search;
mod settings;

use crate::graphql_api::app::graphql_app;
use crate::healthz::healthz_app;
use crate::orgchart::app::orgchart_app;
use crate::search::app::search_app;

use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use dino_park_gate::provider::Provider;
use dino_park_gate::scope::ScopeAndUserAuth;
use failure::Error;
use log::info;

fn main() -> Result<(), Error> {
    ::std::env::set_var(
        "RUST_LOG",
        "actix_web=info,dino_park_fence=info,dino_park_gate=info",
    );
    env_logger::init();
    info!("building the fence");
    let s = settings::Settings::new()?;
    let cis_client = CisClient::from_settings(&s.cis)?;
    let dino_park_settings = s.dino_park.clone();
    let provider = Provider::from_issuer("https://auth.mozilla.auth0.com/")?;
    // Start http server
    HttpServer::new(move || {
        let scope_middleware = ScopeAndUserAuth {
            checker: provider.clone(),
        };
        App::new()
            .wrap(Logger::default().exclude("/healthz"))
            .service(
                web::scope("/api/v4/")
                    .wrap(scope_middleware)
                    .service(graphql_app(cis_client.clone(), &dino_park_settings))
                    .service(search_app(&dino_park_settings.search))
                    .service(orgchart_app(&dino_park_settings.orgchart)),
            )
            .service(healthz_app())
    })
    .bind("0.0.0.0:8081")?
    .run()
    .map_err(Into::into)
}
