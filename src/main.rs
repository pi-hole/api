#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod util;

#[get("/")]
fn index() -> util::Reply {
    util::reply_success()
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
