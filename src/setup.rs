/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Server Setup Functions
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use auth::{self, AuthData};
use config::{Config, PiholeFile};
use dns;
use ftl;
use rocket;
use rocket::config::{ConfigBuilder, Environment};
use rocket::local::Client;
use rocket_cors::{Cors};
use setup_vars::read_setup_vars;
use stats;
use std::collections::HashMap;
use std::fs::File;
use util;
use web;

/// This is run when no route could be found and returns a custom 404 message.
#[error(404)]
fn not_found() -> util::Error {
    util::Error::NotFound
}

#[error(401)]
fn unauthorized() -> util::Error {
    util::Error::Unauthorized
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() {
    let config = Config::Production;
    let key = read_setup_vars("WEBPASSWORD", &config)
        .expect(&format!("Failed to open {}", PiholeFile::SetupVars.default_location()))
        .unwrap_or_default();

    setup(
        rocket::ignite(),
        ftl::FtlConnectionType::Socket,
        config,
        key
    ).launch();
}

/// Setup the API with the testing data and return a Client to test with
pub fn test(
    ftl_data: HashMap<String, Vec<u8>>,
    config_data: HashMap<PiholeFile, File>
) -> Client {
    Client::new(setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Development)
                .finalize()
                .unwrap(),
            false,
        ),
        ftl::FtlConnectionType::Test(ftl_data),
        Config::Test(config_data),
        "test_key".to_owned()
    )).unwrap()
}

/// General server setup
fn setup<'a>(
    server: rocket::Rocket,
    connection_type: ftl::FtlConnectionType,
    config: Config,
    api_key: String
) -> rocket::Rocket {
    // Setup CORS
    let mut cors = Cors::default();
    cors.allow_credentials = true;

    // Start up the server
    server
        // Attach CORS handler
        .attach(cors)
        // Manage the connection type configuration
        .manage(connection_type)
        // Manage the configuration
        .manage(config)
        // Manage the API key
        .manage(AuthData::new(api_key))
        // Mount the web interface
        .mount("/", routes![
            web::web_interface_index,
            web::web_interface
        ])
        // Mount the API
        .mount("/admin/api", routes![
            auth::check,
            auth::logout,
            stats::get_summary,
            stats::top_domains,
            stats::top_domains_params,
            stats::top_blocked,
            stats::top_blocked_params,
            stats::top_clients,
            stats::top_clients_params,
            stats::forward_destinations,
            stats::query_types,
            stats::history,
            stats::history_params,
            stats::recent_blocked,
            stats::recent_blocked_params,
            stats::clients,
            stats::unknown_queries,
            stats::over_time_history,
            stats::over_time_forward_destinations,
            stats::over_time_query_types,
            stats::over_time_clients,
            dns::get_whitelist,
            dns::get_blacklist,
            dns::get_regexlist,
            dns::status,
            dns::add_whitelist,
            dns::add_blacklist,
            dns::add_regexlist,
            dns::delete_whitelist,
            dns::delete_blacklist,
            dns::delete_regexlist
        ])
        // Add custom error handlers
        .catch(errors![not_found, unauthorized])
}
