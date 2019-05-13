// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Server Setup Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::{ftl::FtlDatabase, gravity::GravityDatabase, load_databases},
    env::{Config, Env},
    ftl::{FtlConnectionType, FtlMemory},
    routes::{
        auth::{self, AuthData},
        dns, settings, stats, version, web
    },
    settings::{ConfigEntry, SetupVarsEntry},
    util::{Error, ErrorKind}
};
use rocket::config::{ConfigBuilder, Environment};
use rocket_cors::Cors;

#[cfg(test)]
use crate::databases::load_test_databases;
#[cfg(test)]
use rocket::{config::LoggingLevel, local::Client};
#[cfg(test)]
use std::collections::HashMap;

#[catch(404)]
fn not_found() -> Error {
    Error::from(ErrorKind::NotFound)
}

#[catch(401)]
fn unauthorized() -> Error {
    Error::from(ErrorKind::Unauthorized)
}

/// Run the API normally (connect to FTL over the socket)
pub fn start() -> Result<(), Error> {
    let config = Config::load()?;
    let env = Env::Production(config);
    let key = SetupVarsEntry::WebPassword.read(&env)?;

    setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Production)
                .address(env.config().general.address.as_str())
                .port(env.config().general.port as u16)
                .log_level(env.config().general.log_level)
                .extra("databases", load_databases(&env)?)
                .finalize()
                .unwrap()
        ),
        FtlConnectionType::Socket,
        FtlMemory::production(),
        env,
        key,
        true
    )
    .launch();

    Ok(())
}

/// Setup the API with the testing data and return a Client to test with
#[cfg(test)]
pub fn test(
    ftl_data: HashMap<String, Vec<u8>>,
    ftl_memory: FtlMemory,
    env: Env,
    needs_database: bool
) -> Client {
    Client::new(setup(
        rocket::custom(
            ConfigBuilder::new(Environment::Development)
                .log_level(LoggingLevel::Debug)
                .extra("databases", load_test_databases())
                .finalize()
                .unwrap()
        ),
        FtlConnectionType::Test(ftl_data),
        ftl_memory,
        env,
        "test_key".to_owned(),
        needs_database
    ))
    .unwrap()
}

/// General server setup
fn setup(
    server: rocket::Rocket,
    ftl_socket: FtlConnectionType,
    ftl_memory: FtlMemory,
    env: Env,
    api_key: String,
    needs_database: bool
) -> rocket::Rocket {
    // Set up CORS
    let cors = Cors {
        allow_credentials: true,
        ..Cors::default()
    };

    // Attach the databases if required
    let server = if needs_database {
        server
            .attach(FtlDatabase::fairing())
            .attach(GravityDatabase::fairing())
    } else {
        server
    };

    // Conditionally enable and mount the web interface
    let server = if env.config().web.enabled {
        let web_route = env.config().web.path.to_string_lossy();

        // Check if the root redirect should be enabled
        let server = if env.config().web.root_redirect && web_route != "/" {
            server.mount("/", routes![web::web_interface_redirect])
        } else {
            server
        };

        // Mount the web interface at the configured route
        server.mount(
            &web_route,
            routes![web::web_interface_index, web::web_interface]
        )
    } else {
        server
    };

    // The path to mount the API on (always <web_root>/api)
    let mut api_mount_path = env.config().web.path.clone();
    api_mount_path.push("api");
    let api_mount_path_str = api_mount_path.to_string_lossy();

    // Create a scheduler for scheduling work (ex. disable for 10 minutes)
    let scheduler = task_scheduler::Scheduler::new();

    // Set up the server
    server
        // Attach CORS handler
        .attach(cors)
        // Add custom error handlers
        .register(catchers![not_found, unauthorized])
        // Manage the FTL socket configuration
        .manage(ftl_socket)
        // Manage the FTL shared memory configuration
        .manage(ftl_memory)
        // Manage the environment
        .manage(env)
        // Manage the API key
        .manage(AuthData::new(api_key))
        // Manage the scheduler
        .manage(scheduler)
        // Mount the API
        .mount(&api_mount_path_str, routes![
            version::version,
            auth::check,
            auth::logout,
            stats::get_summary,
            stats::top_domains,
            stats::top_clients,
            stats::upstreams,
            stats::query_types,
            stats::history,
            stats::recent_blocked,
            stats::clients,
            stats::over_time_history,
            stats::over_time_clients,
            stats::database::get_summary_db,
            stats::database::over_time_clients_db,
            stats::database::over_time_history_db,
            stats::database::query_types_db,
            stats::database::top_clients_db,
            stats::database::top_domains_db,
            stats::database::upstreams_db,
            dns::get_whitelist,
            dns::get_blacklist,
            dns::get_regexlist,
            dns::status,
            dns::change_status,
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
            settings::get_network,
            settings::get_web,
            settings::put_web
        ])
}
