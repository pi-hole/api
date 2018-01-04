#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate rmp;

mod util;
mod ftl;

#[get("/stats/summary")]
fn summary() -> util::Reply {
    let mut con = ftl::connect("stats");

    let domains_blocked = con.read_i32().unwrap();
    let total_queries = con.read_i32().unwrap();
    let blocked_queries = con.read_i32().unwrap();
    let percent_blocked = con.read_f32().unwrap();
    let unique_domains = con.read_i32().unwrap();
    let forwarded_queries = con.read_i32().unwrap();
    let cached_queries = con.read_i32().unwrap();
    let total_clients = con.read_i32().unwrap();
    let unique_clients = con.read_i32().unwrap();
    let status = con.read_u8().unwrap();
    con.expect_eom().unwrap();

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
