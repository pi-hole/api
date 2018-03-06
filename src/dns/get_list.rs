/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Endpoints for reading domain lists
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::Config;
use dns::list::{List, get_list};
use rocket::State;
use util;

/// Get the Whitelist domains
#[get("/dns/whitelist")]
pub fn get_whitelist(config: State<Config>) -> util::Reply {
    get_list(List::Whitelist, &config)
}

/// Get the Blacklist domains
#[get("/dns/blacklist")]
pub fn get_blacklist(config: State<Config>) -> util::Reply {
    get_list(List::Blacklist, &config)
}

/// Get the Wildcard list domains
#[get("/dns/wildlist")]
pub fn get_wildlist(config: State<Config>) -> util::Reply {
    get_list(List::Wildlist, &config)
}

#[cfg(test)]
mod test {
    extern crate tempfile;

    use testing::test_endpoint;
    use config::PiholeFile;
    use std::collections::HashMap;
    use rocket::http::Method;
    use std::io::prelude::*;
    use std::io::SeekFrom;

    #[test]
    fn test_get_whitelist() {
        let mut whitelist = tempfile::tempfile().unwrap();
        let mut setup_vars = tempfile::tempfile().unwrap();

        writeln!(whitelist, "{}", ["example.com", "example.net"].join("\n")).unwrap();
        writeln!(setup_vars, "IPV4_ADDRESS=10.1.1.1").unwrap();

        whitelist.seek(SeekFrom::Start(0)).unwrap();
        setup_vars.seek(SeekFrom::Start(0)).unwrap();

        let mut data = HashMap::new();
        data.insert(PiholeFile::Whitelist, whitelist.try_clone().unwrap());
        data.insert(PiholeFile::SetupVars, setup_vars.try_clone().unwrap());

        test_endpoint(
            Method::Get,
            "/admin/api/dns/whitelist",
            HashMap::default(),
            data,
            json!({
                "data": [
                    "example.com",
                    "example.net"
                ],
                "errors": []
            })
        );

        // todo: verify whitelist and setupvars aren't changed
    }

    #[test]
    fn test_get_blacklist() {
        let mut blacklist = tempfile::tempfile().unwrap();
        let mut setup_vars = tempfile::tempfile().unwrap();

        writeln!(blacklist, "{}", ["example.com", "example.net"].join("\n")).unwrap();
        writeln!(setup_vars, "IPV4_ADDRESS=10.1.1.1").unwrap();

        blacklist.seek(SeekFrom::Start(0)).unwrap();
        setup_vars.seek(SeekFrom::Start(0)).unwrap();

        let mut data = HashMap::new();
        data.insert(PiholeFile::Blacklist, blacklist.try_clone().unwrap());
        data.insert(PiholeFile::SetupVars, setup_vars.try_clone().unwrap());

        test_endpoint(
            Method::Get,
            "/admin/api/dns/blacklist",
            HashMap::default(),
            data,
            json!({
                "data": [
                    "example.com",
                    "example.net"
                ],
                "errors": []
            })
        );
    }

    #[test]
    fn test_get_wildlist() {
        let mut wildlist = tempfile::tempfile().unwrap();
        let mut setup_vars = tempfile::tempfile().unwrap();

        writeln!(
            wildlist,
            "{}",
            [
                "address=/example.com/10.1.1.1",
                "address=/example.net/10.1.1.1"
            ].join("\n")
        ).unwrap();
        writeln!(setup_vars, "IPV4_ADDRESS=10.1.1.1").unwrap();

        wildlist.seek(SeekFrom::Start(0)).unwrap();
        setup_vars.seek(SeekFrom::Start(0)).unwrap();

        let mut data = HashMap::new();
        data.insert(PiholeFile::Wildlist, wildlist.try_clone().unwrap());
        data.insert(PiholeFile::SetupVars, setup_vars.try_clone().unwrap());

        test_endpoint(
            Method::Get,
            "/admin/api/dns/wildlist",
            HashMap::default(),
            data,
            json!({
                "data": [
                    "example.com",
                    "example.net"
                ],
                "errors": []
            })
        );
    }
}
