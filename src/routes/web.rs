// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Web Interface Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use rocket::{
    http::ContentType,
    response::{Redirect, Response}
};
use std::{borrow::Cow, io::Cursor, path::PathBuf};

#[derive(RustEmbed)]
#[folder = "web/"]
pub struct WebAssets;

/// Get a file from the embedded web assets
fn get_file<'r>(filename: &str) -> Option<Response<'r>> {
    let has_extension = filename.contains('.');
    let content_type = if has_extension {
        match ContentType::from_extension(filename.rsplit('.').next().unwrap()) {
            Some(value) => value,
            None => return None
        }
    } else {
        ContentType::Binary
    };

    WebAssets::get(filename).map_or_else(
        // If the file was not found, and there is no extension on the filename,
        // fall back to the web interface index.html
        || {
            if !has_extension {
                WebAssets::get("index.html").map(|data| build_response(data, ContentType::HTML))
            } else {
                None
            }
        },
        // The file was found, so build the response
        |data| Some(build_response(data, content_type))
    )
}

/// Build a `Response` from raw data and its content type
fn build_response<'r>(data: Cow<'static, [u8]>, content_type: ContentType) -> Response<'r> {
    Response::build()
        .header(content_type)
        .sized_body(Cursor::new(data))
        .finalize()
}

/// Redirect root requests to the web interface. This allows http://pi.hole to
/// redirect to http://pi.hole/admin
#[get("/")]
pub fn web_interface_redirect() -> Redirect {
    Redirect::to(uri!(web_interface_index))
}

/// Return the index page of the web interface. This handler is mounted on a
/// route taken from the config, so it must use `/`.
#[get("/")]
pub fn web_interface_index<'r>() -> Option<Response<'r>> {
    get_file("index.html")
}

/// Return the requested page/file, if it exists. This handler is mounted on a
/// route taken from the config, so it must use `/`.
#[get("/<path..>")]
pub fn web_interface<'r>(path: PathBuf) -> Option<Response<'r>> {
    get_file(&path.display().to_string())
}
