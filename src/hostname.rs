/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Local hostname
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

extern crate hostname;

use util;

/// Get local host name
#[get("/hostname")]
pub fn hostname() -> util::Reply {
    match hostname::get_hostname() {
        None => {
            return util::reply_data(json!({
                "hostname": ""
            }));
        }
        Some(h) => {
            return util::reply_data(json!({
                "hostname": h
            }));
        }
    }
}

