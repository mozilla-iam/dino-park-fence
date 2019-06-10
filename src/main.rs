extern crate actix_web;
extern crate biscuit;
extern crate chrono;
extern crate cis_client;
extern crate cis_profile;
extern crate condvar_store;
extern crate config;
extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate percent_encoding;
extern crate reqwest;
extern crate serde;
extern crate url;

#[macro_use]
extern crate juniper;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod graphql_api;
mod healthz;
mod orgchart;
mod permissions;
mod proxy;
mod search;
mod settings;
mod timezones;

use crate::graphql_api::app::graphql_app;
use crate::healthz::healthz_app;
use crate::orgchart::app::orgchart_app;
use crate::search::app::search_app;
use crate::timezones::app::timezone_app;

use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use failure::Error;

fn main() -> Result<(), Error> {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dino_park_fence=info");
    env_logger::init();
    info!("building the fence");
    let s = settings::Settings::new()?;
    let cis_client = CisClient::from_settings(&s.cis)?;
    let dino_park_settings = s.dino_park.clone();

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default().exclude("/healthz"))
            .service(
                web::scope("/api/v4/")
                    .service(graphql_app(cis_client.clone(), &dino_park_settings))
                    .service(search_app(&dino_park_settings.search))
                    .service(orgchart_app(&dino_park_settings.orgchart))
                    .service(timezone_app()),
            )
            .service(healthz_app())
    })
    .bind("0.0.0.0:8081")?
    .run()
    .map_err(Into::into)
}
