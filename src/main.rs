extern crate actix;
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
mod search;
mod settings;
mod timezones;

use crate::graphql_api::app::graphql_app;
use crate::healthz::healthz_app;
use crate::orgchart::app::orgchart_app;
use crate::search::app::search_app;
use crate::timezones::app::timezone_app;

use actix_web::middleware;
use actix_web::server;
use cis_client::client::CisClient;

fn main() -> Result<(), String> {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dino_park_fence=info");
    env_logger::init();
    info!("building the fence");
    let sys = actix::System::new("dino-park-fence");
    let s = settings::Settings::new().map_err(|e| format!("unable to load settings: {}", e))?;
    let cis_client = CisClient::from_settings(&s.cis)
        .map_err(|e| format!("unable to create cis_client: {}", e))?;
    let dino_park_settings = s.dino_park.clone();

    // Start http server
    server::new(move || {
        vec![
            search_app(&dino_park_settings.search)
                .middleware(middleware::Logger::default())
                .boxed(),
            orgchart_app(&dino_park_settings.orgchart)
                .middleware(middleware::Logger::default())
                .boxed(),
            graphql_app(cis_client.clone(), &dino_park_settings)
                .middleware(middleware::Logger::default())
                .boxed(),
            timezone_app()
                .middleware(middleware::Logger::default())
                .boxed(),
            healthz_app()
                .boxed(),
        ]
    })
    .bind("0.0.0.0:8081")
    .unwrap()
    .start();

    info!("Started http server");
    let _ = sys.run();
    Ok(())
}
