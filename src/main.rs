#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod util;
mod ftl;

#[get("/stats/summary")]
fn summary() -> util::Reply {
    let stream = ftl::connect("stats");

    for line in stream {
        println!("Received from FTL: {}", line);
    }

    util::reply_success()
}

fn main() {
    rocket::ignite().mount("/", routes![summary]).launch();
}
