/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  FTL Information - db stats
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util::{Reply, reply_data};
use auth::User;

/// Read memory and db stats from FTL
#[get("/settings/ftldb")]
pub fn ftldb(ftl: State<FtlConnectionType>, _auth: User) -> Reply {
    let mut con = ftl.connect("dbstats")?;
    // Read in FTL's database stats
    let db_queries = con.read_i32()?;
    let db_filesize = con.read_i64()?;
    let mut version_buffer = [0u8; 64];
    let db_sqlite_version = con.read_str(&mut version_buffer)?;  
    con.expect_eom()?;

    reply_data(json!({
        "queries": db_queries,
        "filesize": db_filesize,
        "sqlite_version": db_sqlite_version
    }))
}


/*
use failure::ResultExt;
use ftl::FtlConnectionType;
use std::io::Read;
use std::str;

use routes::web::WebAssets;
*/

#[cfg(test)]
mod tests {
    use testing::{TestEnvBuilder, write_eom};
    use ftl::FtlConnectionType;
    use rmp::encode;
    use std::collections::HashMap;

    #[test]
    fn test_ftldb() {
    let mut data = Vec::new();
    encode::write_i32(&mut data, 340934).unwrap();
    encode::write_i64(&mut data, 85843).unwrap();
    encode::write_str(&mut data, "3.0.1").unwrap();
    write_eom(&mut data);

    let mut map = HashMap::new();
    map.insert("debstats".to_owned(), data);

    let ftl = FtlConnectionType::Test(map);

    assert_eq!(
        ftldb(&ftl).map_err(|e| e.kind()),
        Ok(json!({
            "queries": 340934,
            "filesize": 85843,
            "sqlite_version": "3.0.1"
        }))
    )
    }
}
            








