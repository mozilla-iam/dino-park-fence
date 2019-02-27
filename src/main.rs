extern crate actix;
extern crate actix_web;
extern crate biscuit;
extern crate chrono;
extern crate cis_profile;
extern crate config;
extern crate env_logger;
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
#[macro_use]
extern crate serde_json;

mod cis;
mod graphql_api;
mod remote_store;
mod settings;

use crate::graphql_api::app::graphql_app;

use actix_web::middleware;
use actix_web::server;

fn main() -> Result<(), String> {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dino_park_fence=info");
    env_logger::init();
    info!("building the fence");
    let sys = actix::System::new("juniper-example");
    let s = settings::Settings::new().map_err(|e| format!("unable to load settings: {}", e))?;
    let cis_client = cis::client::CisClient::from_settings(&s)?;

    // Start http server
    server::new(move || {
        vec![graphql_app(cis_client.clone())
            .middleware(middleware::Logger::default())
            .boxed()]
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .start();

    info!("Started http server");
    let _ = sys.run();
    Ok(())
}
