/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  General API Utilities
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use serde::Serialize;
use rocket_contrib::{Json, Value};
use rocket::request::Request;
use rocket::response::{Response, Responder};
use std::fmt::Display;

pub type Reply = Result<CORS<Json<Value>>, Error>;

pub fn reply<D: Serialize>(data: D, errors: &[Error]) -> Reply {
    Ok(CORS(Json(json!({
        "data": data,
        "errors": errors.iter()
                        .map(|error| json!({
                            "key": error.key(),
                            "message": error.message()
                        }))
                        .collect::<Vec<Value>>()
    }))))
}

pub fn reply_data<D: Serialize>(data: D) -> Reply {
    reply(data, &[])
}

pub fn reply_error(error: Error) -> Reply {
    reply([0; 0], &[error])
}

pub fn reply_success() -> Reply {
    reply(json!({
        "status": "success"
    }), &[])
}

#[derive(Debug)]
pub enum Error {
    Unknown,
    Custom(String),
    FtlConnectionFail,
    AlreadyExists,
    NotFound
}

impl Error {
    pub fn message(&self) -> &str {
        match *self {
            Error::Unknown => "Unknown error",
            Error::Custom(ref msg) => msg,
            Error::FtlConnectionFail => "Failed to connect to FTL",
            Error::AlreadyExists => "Item already exists",
            Error::NotFound => "Not found"
        }
    }

    pub fn key(&self) -> &str {
        match *self {
            Error::Unknown => "unknown",
            Error::Custom(_) => "custom",
            Error::FtlConnectionFail => "ftl_connection_fail",
            Error::AlreadyExists => "already_exists",
            Error::NotFound => "not_found"
        }
    }
}

impl<T: Display> From<T> for Error {
    fn from(e: T) -> Self {
        Error::Custom(format!("{}", e))
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &Request) -> super::rocket::response::Result<'r> {
        reply_error(self).unwrap().respond_to(request)
    }
}

#[derive(Debug)]
pub struct CORS<R>(R);

impl<'r, R: Responder<'r>> Responder<'r> for CORS<R> {
    fn respond_to(self, request: &Request) -> super::rocket::response::Result<'r> {
        Ok(Response::build_from(self.0.respond_to(request)?)
            .raw_header("Access-Control-Allow-Origin", "*")
            .finalize())
    }
}
