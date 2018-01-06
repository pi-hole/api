#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
extern crate rmp;

mod util;
mod ftl;
mod stats;

fn main() {
    rocket::ignite()
        .mount("/", routes![
            stats::summary,
            stats::over_time
        ])
        .launch();
}
