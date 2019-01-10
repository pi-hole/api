// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Program Main
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

fn main() {
    if let Err(e) = pihole_api::start() {
        e.print_stacktrace();
    }
}
