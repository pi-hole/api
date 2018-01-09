#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmp;

mod util;
mod ftl;
mod stats;
mod web;

fn main() {
    rocket::ignite()
        .mount("/", routes![
            web::web_interface_index,
            web::web_interface,
            stats::summary,
            stats::top_domains,
            stats::top_domains_params,
            stats::top_blocked,
            stats::top_blocked_params,
            stats::top_clients,
            stats::forward_destinations,
            stats::query_types,
            stats::history,
            stats::recent_blocked,
            stats::clients,
            stats::unknown_queries,
            stats::over_time_history,
            stats::over_time_forward_destinations,
            stats::over_time_query_types,
            stats::over_time_clients
        ])
        .catch(errors![not_found])
        .launch();
}

#[error(404)]
fn not_found() -> util::Reply {
    util::reply_error(util::Error::NotFound)
}
