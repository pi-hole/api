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

fn main() {
    rocket::ignite()
        .mount("/", routes![
            stats::summary,
            stats::over_time,
            stats::top_domains,
            stats::top_blocked,
            stats::top_clients,
            stats::forward_destinations,
            stats::query_types,
            stats::history
        ])
        .launch();
}
