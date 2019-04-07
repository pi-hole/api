// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Web Interface Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::env::Env;
use rocket::{
    http::ContentType,
    response::{Redirect, Response},
    State
};
use std::{borrow::Cow, io::Cursor, path::PathBuf};

#[derive(RustEmbed)]
#[folder = "web/"]
pub struct WebAssets;

/// Get a file from the embedded web assets
fn get_file<'r>(filename: &str, env: &Env) -> Option<Response<'r>> {
    // The default is index.html, and it requires special handling
    if filename.is_empty() || filename == "index.html" {
        return get_index_response(env);
    }

    // The file is not index.html, so find out its content type
    let has_extension = filename.contains('.');
    let content_type = if has_extension {
        match ContentType::from_extension(filename.rsplit('.').next().unwrap()) {
            Some(value) => value,
            None => ContentType::Binary
        }
    } else {
        ContentType::Binary
    };

    // Get the file from the assets
    match WebAssets::get(filename) {
        // The file was found, so build the response
        Some(data) => Some(build_response(data, content_type)),
        // If the file was not found, and there is no extension on the filename,
        // fall back to the web interface index.html
        None => {
            if !has_extension {
                get_index_response(env)
            } else {
                None
            }
        }
    }
}

/// Build a `Response` from raw data and its content type
fn build_response<'r>(data: Cow<'static, [u8]>, content_type: ContentType) -> Response<'r> {
    Response::build()
        .header(content_type)
        .sized_body(Cursor::new(data))
        .finalize()
}

/// Get index.html and build a response for it
fn get_index_response<'r>(env: &Env) -> Option<Response<'r>> {
    get_index_html(env).map(|data| build_response(data, ContentType::HTML))
}

/// Get index.html and inject a `<base>` element into the `<head>` element.
/// This will tell the web interface where it is mounted so that it can use the
/// correct paths when loading its resources. This is required because the web
/// interface could be mounted from anywhere, so it uses relative paths.
///
/// For example, if `/admin/settings/network` was loaded, without this base
/// element the web interface would try to load `./main.js` which resolves to
/// `/admin/settings/main.js`. This fails because that file does not exist. With
/// a base element of `<base href='/admin/'>`, it would correctly load
/// `/admin/main.js`.
///
/// (`main.js` in a real scenario would have a more complicated name and path,
/// such as `static/js/main.9e23e19a.chunk.js`)
fn get_index_html(env: &Env) -> Option<Cow<'static, [u8]>> {
    // Get index.html as a string
    let index_bytes = WebAssets::get("index.html")?;
    let mut index_string = String::from_utf8(index_bytes.into_owned()).ok()?;

    // Find the location and length of the head element
    let head_index = index_string.find("<head>")?;
    let length_of_head = "<head>".len();

    // Configure the base element
    let base_path = env.config().web.path_with_trailing_slash();
    let base_element = format!("<base href='{}'>", base_path);

    // Inject the base element into index.html after the head element
    index_string.insert_str(head_index + length_of_head, &base_element);

    Some(Cow::Owned(index_string.into_bytes()))
}

/// Redirect root requests to the web interface. This allows http://pi.hole to
/// redirect to http://pi.hole/admin
#[get("/")]
pub fn web_interface_redirect(env: State<Env>) -> Redirect {
    Redirect::to(env.config().web.path.to_string_lossy().into_owned())
}

/// Return the index page of the web interface. This handler is mounted on a
/// route taken from the config, such as `/admin`, so it must use `/`.
#[get("/")]
pub fn web_interface_index<'r>(env: State<Env>) -> Option<Response<'r>> {
    get_index_response(&env)
}

/// Return the requested page/file, if it exists. This handler is mounted on a
/// route taken from the config, such as `/admin`, so it must use `/`.
#[get("/<path..>")]
pub fn web_interface<'r>(path: PathBuf, env: State<Env>) -> Option<Response<'r>> {
    get_file(&path.display().to_string(), &env)
}
