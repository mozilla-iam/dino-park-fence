extern crate actix;
extern crate actix_web;
extern crate biscuit;
extern crate chrono;
extern crate cis_profile;
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
mod config;
mod graphql_api;
mod remote_store;

use crate::cis::auth;
use crate::cis::secrets::get_store_from_ssm_via_env;
use crate::config::Config;
use crate::graphql_api::app::graphql_app;

use actix_web::middleware;
use actix_web::server;

use std::sync::Arc;

fn main() -> Result<(), String> {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dino_park_fence=info");
    env_logger::init();
    info!("building the fence");
    let sys = actix::System::new("juniper-example");
    let client_config = auth::read_client_config(".person-api.json")?;
    let bearer_store = remote_store::RemoteStore::new(auth::BaererBaerer::new(client_config));
    let secret_store = Arc::new(get_store_from_ssm_via_env()?);
    let cfg = Config { bearer_store, secret_store };

    // Start http server
    server::new(move || {
        vec![
            graphql_app(cfg.clone())
                .middleware(middleware::Logger::default())
                .boxed(),
        ]
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    info!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
    Ok(())
}
