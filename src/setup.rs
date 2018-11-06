// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Server Setup Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::{self, AuthData};
use env::{Config, Env};
use ftl::{FtlConnectionType, FtlMemory};
use rocket::{
    self,
    config::{ConfigBuilder, Environment}
};
use rocket_cors::Cors;
use routes::{dns, settings, stats, version, web};
use settings::{ConfigEntry, SetupVarsEntry};
use util::{Error, ErrorKind};

#[cfg(test)]
use env::PiholeFile;
#[cfg(test)]
use rocket::local::Client;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use tempfile::NamedTempFile;

const CONFIG_LOCATION: &'static str = "/etc/pihole/API.toml";

#[error(404)]
fn not_found() -> Error {
    Error::from(ErrorKind::NotFound)
}

#[error(401)]
fn unauthorized() -> Error {
    Error::from(ErrorKind::Unauthorized)
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() -> Result<(), Error> {
    let config = Config::parse(CONFIG_LOCATION)?;
    let env = Env::Production(config);
    let key = SetupVarsEntry::WebPassword.read(&env)?;

    setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Production)
                .address(env.config().address())
                .port(env.config().port() as u16)
                .log_level(env.config().log_level())
                .finalize()
                .unwrap(),
            // TODO: Add option to turn off logs
            true
        ),
        FtlConnectionType::Socket,
        FtlMemory::Production,
        env,
        key
    ).launch();

    Ok(())
}

/// Setup the API with the testing data and return a Client to test with
#[cfg(test)]
pub fn test(
    ftl_data: HashMap<String, Vec<u8>>,
    ftl_memory: FtlMemory,
    env_data: HashMap<PiholeFile, NamedTempFile>
) -> Client {
    use toml;

    Client::new(setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Development)
                .finalize()
                .unwrap(),
            false
        ),
        FtlConnectionType::Test(ftl_data),
        ftl_memory,
        Env::Test(toml::from_str("").unwrap(), env_data),
        "test_key".to_owned()
    )).unwrap()
}

/// General server setup
fn setup(
    server: rocket::Rocket,
    ftl_socket: FtlConnectionType,
    ftl_memory: FtlMemory,
    env: Env,
    api_key: String
) -> rocket::Rocket {
    // Set up CORS
    let cors = Cors {
        allow_credentials: true,
        ..Cors::default()
    };

    // Set up the server
    server
        // Attach CORS handler
        .attach(cors)
        // Manage the FTL socket configuration
        .manage(ftl_socket)
        // Manage the FTL shared memory configuration
        .manage(ftl_memory)
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
            stats::top_clients,
            stats::top_clients_params,
            stats::upstreams,
            stats::query_types,
            stats::history,
            stats::history_params,
            stats::recent_blocked,
            stats::recent_blocked_params,
            stats::clients,
            stats::clients_params,
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
            settings::put_dhcp,
            settings::get_dns,
            settings::put_dns,
            settings::get_ftldb,
            settings::get_ftl,
            settings::get_network
        ])
        // Add custom error handlers
        .catch(errors![not_found, unauthorized])
}
