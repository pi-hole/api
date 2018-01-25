/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Web Interface Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

/// Return the index page of the web interface
#[get("/admin")]
pub fn web_interface_index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").ok()
}

/// Return the requested page/file, if it exists. This automatically handles preventing path
/// traversal attacks.
/// See https://rocket.rs/guide/requests/#multiple-segments
/// or https://api.rocket.rs/rocket/request/trait.FromSegments.html#provided-implementations
#[get("/admin/<path..>")]
pub fn web_interface(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}