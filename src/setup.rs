/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Server Setup Functions
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::{Config, PiholeFile};
use dns;
use ftl;
use rocket;
use rocket::config::{ConfigBuilder, Environment};
use rocket::local::Client;
use stats;
use std::collections::HashMap;
use std::fs::File;
use util;
use web;

/// This is run when no route could be found and returns a custom 404 message.
#[error(404)]
fn not_found() -> util::Reply {
    util::reply_error(util::Error::NotFound)
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() {
    setup(
        rocket::ignite(),
        ftl::FtlConnectionType::Socket,
        Config::Production
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
        Config::Test(config_data)
    )).unwrap()
}

/// General server setup
fn setup<'a>(
    server: rocket::Rocket,
    connection_type: ftl::FtlConnectionType,
    config: Config
) -> rocket::Rocket {
    // Start up the server
    server
        // Manage the connection type configuration
        .manage(connection_type)
        // Manage the configuration
        .manage(config)
        // Mount the web interface
        .mount("/", routes![
            web::web_interface_index,
            web::web_interface
        ])
        // Mount the API
        .mount("/admin/api", routes![
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
            stats::history_timespan,
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
            dns::get_wildlist,
            dns::status,
            dns::add_whitelist,
            dns::add_blacklist,
            dns::add_wildlist,
            dns::delete_whitelist,
            dns::delete_blacklist,
            dns::delete_wildlist
        ])
        // Add custom error handlers
        .catch(errors![not_found])
}
