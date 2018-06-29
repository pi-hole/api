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
use config::{Env, PiholeFile, Config};
use ftl::FtlConnectionType;
use rocket;
use rocket::config::{ConfigBuilder, Environment};
use rocket::local::Client;
use rocket_cors::{Cors};
use setup_vars::read_setup_vars;
use routes::{dns, settings, stats, version, web};
use std::collections::HashMap;
use std::fs::File;
use toml;
use util::{Error, ErrorKind};

const CONFIG_LOCATION: &'static str = "/etc/pihole/API.toml";

#[error(404)]
fn not_found() -> Error {
    ErrorKind::NotFound.into()
}

#[error(401)]
fn unauthorized() -> Error {
    ErrorKind::Unauthorized.into()
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() -> Result<(), Error> {
    let config = Config::parse(CONFIG_LOCATION)?;
    let env = Env::Production(config);
    let key = read_setup_vars("WEBPASSWORD", &env)?.unwrap_or_default();

    setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Production)
                .address(env.config().address())
                .port(env.config().port() as u16)
                .log_level(env.config().log_level())
                .finalize().unwrap(),
            // TODO: Add option to turn off logs
            true
        ),
        FtlConnectionType::Socket,
        env,
        key
    ).launch();

    Ok(())
}

/// Setup the API with the testing data and return a Client to test with
pub fn test(
    ftl_data: HashMap<String, Vec<u8>>,
    env_data: HashMap<PiholeFile, File>
) -> Client {
    Client::new(setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Development)
                .finalize()
                .unwrap(),
            false,
        ),
        FtlConnectionType::Test(ftl_data),
        Env::Test(toml::from_str("").unwrap(), env_data),
        "test_key".to_owned()
    )).unwrap()
}

/// General server setup
fn setup<'a>(
    server: rocket::Rocket,
    connection_type: FtlConnectionType,
    env: Env,
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
        // Manage the environment
        .manage(env)
        // Manage the API key
        .manage(AuthData::new(api_key))
        // Mount the web interface
        .mount("/", routes![
            web::web_interface_index,
            web::web_interface
        ])
        // Mount the API
        .mount("/admin/api", routes![
            version::version,
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
            dns::delete_regexlist,
            settings::get_dhcp,
            settings::get_dns,
            settings::get_ftldb,
            settings::get_network
        ])
        // Add custom error handlers
        .catch(errors![not_found, unauthorized])
}
