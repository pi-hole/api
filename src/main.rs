// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Program Main
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use std::env;

fn main() {
    // Collect all arguments except for the first, which is the program name
    let args: Vec<String> = env::args().skip(1).collect();

    // Handle the arguments if any
    if !args.is_empty() {
        handle_args(args);
        return;
    }

    // Only run the API if there are no arguments
    if let Err(e) = pihole_api::start() {
        e.print_stacktrace();
    }
}

fn handle_args(args: Vec<String>) {
    // There should only be one argument
    if args.len() != 1 {
        print_usage();
        return;
    }

    match args[0].as_str() {
        "version" => print_version(),
        "branch" => println!("{}", env!("GIT_BRANCH")),
        "hash" => println!("{}", get_hash()),
        "help" | _ => print_usage()
    }
}

fn print_version() {
    let tag = env!("GIT_TAG");

    if !tag.is_empty() {
        println!("{}", tag);
    } else {
        println!("vDev-{}", get_hash());
    }
}

fn print_usage() {
    let program_name = env::args()
        .next()
        .unwrap_or_else(|| "pihole-API".to_owned());

    println!("Usage: {} [version | branch | hash | help]", program_name);
}

fn get_hash() -> &'static str {
    env!("GIT_HASH").get(0..7).unwrap_or_default()
}
