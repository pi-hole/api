#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmp;

mod util;
mod ftl;

use rmp::decode;

#[get("/stats/summary")]
fn summary() -> util::Reply {
    let mut stream = ftl::connect("stats");

    let domains_blocked = decode::read_i32(&mut stream).unwrap();
    let total_queries = decode::read_i32(&mut stream).unwrap();
    let blocked_queries = decode::read_i32(&mut stream).unwrap();
    let percent_blocked = decode::read_f32(&mut stream).unwrap();
    let unique_domains = decode::read_i32(&mut stream).unwrap();
    let forwarded_queries = decode::read_i32(&mut stream).unwrap();
    let cached_queries = decode::read_i32(&mut stream).unwrap();
    let total_clients = decode::read_i32(&mut stream).unwrap();
    let unique_clients = decode::read_i32(&mut stream).unwrap();
    let status = decode::read_u8(&mut stream).unwrap();
    ftl::expect_eom(&mut stream).unwrap();

    util::reply_data(json!({
        "domains_blocked": domains_blocked,
        "total_queries": total_queries,
        "blocked_queries": blocked_queries,
        "percent_blocked": percent_blocked,
        "unique_domains": unique_domains,
        "forwarded_queries": forwarded_queries,
        "cached_queries": cached_queries,
        "total_clients": total_clients,
        "unique_clients": unique_clients,
        "status": status
    }))
}

fn main() {
    rocket::ignite().mount("/", routes![summary]).launch();
}
