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
use rocket::response::{self, Response, Responder};
use rocket::http::Status;
use std::fmt::Display;

pub type Reply = Result<CORS<SetStatus<Json<Value>>>, Error>;

pub fn reply<D: Serialize>(data: D, errors: &[Error], status: Status) -> Reply {
    Ok(CORS(SetStatus(Json(json!({
        "data": data,
        "errors": errors.iter()
                        .map(|error| json!({
                            "key": error.key(),
                            "message": error.message()
                        }))
                        .collect::<Vec<Value>>()
    })), status)))
}

pub fn reply_data<D: Serialize>(data: D) -> Reply {
    reply(data, &[], Status::Ok)
}

pub fn reply_error(error: Error) -> Reply {
    let status = error.status();
    reply([0; 0], &[error], status)
}

pub fn reply_success() -> Reply {
    reply(json!({
        "status": "success"
    }), &[], Status::Ok)
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Unknown,
    GravityError,
    Custom(String, Status),
    FtlConnectionFail,
    NotFound,
    AlreadyExists,
    InvalidDomain
}

impl Error {
    pub fn message(&self) -> &str {
        match *self {
            Error::Unknown => "Unknown error",
            Error::GravityError => "Gravity failed to form",
            Error::Custom(ref msg, _) => msg,
            Error::FtlConnectionFail => "Failed to connect to FTL",
            Error::NotFound => "Not found",
            Error::AlreadyExists => "Item already exists",
            Error::InvalidDomain => "Bad request"
        }
    }

    pub fn key(&self) -> &str {
        match *self {
            Error::Unknown => "unknown",
            Error::GravityError => "gravity_error",
            Error::Custom(_, _) => "custom",
            Error::FtlConnectionFail => "ftl_connection_fail",
            Error::NotFound => "not_found",
            Error::AlreadyExists => "already_exists",
            Error::InvalidDomain => "invalid_domain"
        }
    }

    pub fn status(&self) -> Status {
        match *self {
            Error::Unknown => Status::InternalServerError,
            Error::GravityError => Status::InternalServerError,
            Error::Custom(_, status) => status,
            Error::FtlConnectionFail => Status::InternalServerError,
            Error::NotFound => Status::NotFound,
            Error::AlreadyExists => Status::Conflict,
            Error::InvalidDomain => Status::BadRequest
        }
    }
}

impl<T: Display> From<T> for Error {
    fn from(e: T) -> Self {
        Error::Custom(format!("{}", e), Status::InternalServerError)
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        reply_error(self).unwrap().respond_to(request)
    }
}

#[derive(Debug)]
pub struct CORS<R>(R);

impl<'r, R: Responder<'r>> Responder<'r> for CORS<R> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        Ok(Response::build_from(self.0.respond_to(request)?)
            .raw_header("Access-Control-Allow-Origin", "*")
            .finalize())
    }
}

#[derive(Debug)]
pub struct SetStatus<R>(R, Status);

impl<'r, R: Responder<'r>> Responder<'r> for SetStatus<R> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        Ok(Response::build_from(self.0.respond_to(request)?)
            .status(self.1)
            .finalize())
    }
}
