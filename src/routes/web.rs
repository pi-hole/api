// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Web Interface Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use rocket::{http::ContentType, response::Response};
use std::{io::Cursor, path::PathBuf};

#[derive(RustEmbed)]
#[folder = "web/"]
pub struct WebAssets;

/// Get a file from the embedded web assets
fn get_file<'r>(filename: &str) -> Option<Response<'r>> {
    let content_type = if filename.contains(".") {
        match ContentType::from_extension(filename.rsplit(".").next().unwrap()) {
            Some(value) => value,
            None => return None
        }
    } else {
        ContentType::Binary
    };

    WebAssets::get(filename).map_or_else(
        || None,
        |data| {
            Some(
                Response::build()
                    .header(content_type)
                    .sized_body(Cursor::new(data))
                    .finalize()
            )
        }
    )
}

/// Return the index page of the web interface
#[get("/admin")]
pub fn web_interface_index<'r>() -> Option<Response<'r>> {
    get_file("index.html")
}

/// Return the requested page/file, if it exists.
#[get("/admin/<path..>")]
pub fn web_interface<'r>(path: PathBuf) -> Option<Response<'r>> {
    get_file(&path.display().to_string())
}
