/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Root Library File
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate regex;
extern crate rmp;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use rocket::local::Client;
use rocket::config::{ConfigBuilder, Environment};

mod util;
mod ftl;
mod stats;
mod dns;
mod web;

/// This is run when no route could be found and returns a custom 404 message.
#[error(404)]
fn not_found() -> util::Reply {
    util::reply_error(util::Error::NotFound)
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() {
    setup(rocket::ignite(), ftl::FtlConnectionType::Socket).launch();
}

/// Setup the API with the testing data and return a Client to test with
pub fn test(test_data: HashMap<String, Vec<u8>>) -> Client {
    Client::new(setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Development)
                .finalize()
                .unwrap(),
            false,
        ),
        ftl::FtlConnectionType::Test(test_data),
    )).unwrap()
}

/// General Rocket setup
fn setup<'a>(server: rocket::Rocket, connection_type: ftl::FtlConnectionType) -> rocket::Rocket {
    // Start up the server
    server
        // Manage the connection type configuration
        .manage(connection_type)
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
