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

