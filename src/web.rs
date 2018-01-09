use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

#[get("/admin")]
pub fn web_interface_index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").ok()
}

#[get("/admin/<path..>")]
pub fn web_interface(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}