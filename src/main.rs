#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod util;
mod ftl;

use std::io::prelude::*;
use std::io::BufReader;

#[get("/stats/summary")]
fn summary() -> util::Reply {
    let mut stream = ftl::connect();
    let mut data = String::new();

    stream.write_all(b">stats\n").unwrap();

    let mut reader = BufReader::new(stream);

    while reader.read_line(&mut data).is_ok() {
        println!("{:?}", data);

        if data.contains("---EOM---") {
            break;
        }

        data.clear();
    }

    util::reply_success()
}

fn main() {
    rocket::ignite().mount("/", routes![summary]).launch();
}
