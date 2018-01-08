#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmp;

mod util;
#[macro_use] mod ftl;
mod stats;
mod web;

fn main() {
    rocket::ignite()
        .mount("/", routes![
            web::index,
            stats::summary,
            stats::top_domains,
            stats::top_blocked,
            stats::top_clients,
            stats::forward_destinations,
            stats::query_types,
            stats::history,
            stats::recent_blocked,
            stats::over_time_history,
            stats::over_time_forward_destinations,
            stats::over_time_query_types
        ])
        .launch();
}
