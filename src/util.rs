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
use rocket::{Request, Outcome};
use rocket::request;
use rocket::response::{self, Response, Responder};
use rocket::http::Status;
use std::fmt::Display;

/// Type alias for the most common return type of the API methods
pub type Reply = Result<SetStatus<Json<Value>>, Error>;

/// The most general reply builder. It takes in data, errors, and status and constructs the JSON
/// reply.
pub fn reply<D: Serialize>(data: D, errors: &[Error], status: Status) -> Reply {
    Ok(SetStatus(Json(json!({
        "data": data,
        "errors": errors.iter()
                        .map(|error| json!({
                            "key": error.key(),
                            "message": error.message()
                        }))
                        .collect::<Vec<Value>>()
    })), status))
}

/// Create a reply from some serializable data. The reply will contain no errors and will have a
/// status code of 200.
pub fn reply_data<D: Serialize>(data: D) -> Reply {
    reply(data, &[], Status::Ok)
}

/// Create a reply with an error. The data will be an empty array and the status will taken from
/// `error.status()`.
pub fn reply_error(error: Error) -> Reply {
    let status = error.status();
    reply([0; 0], &[error], status)
}

/// Create a reply with a successful status. There are no errors and the status is 200.
pub fn reply_success() -> Reply {
    reply(json!({
        "status": "success"
    }), &[], Status::Ok)
}

/// The `Error` enum represents all the possible errors that the API can return. These errors have
/// messages, keys, and HTTP statuses.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Unknown,
    GravityError,
    Custom(String, Status),
    FtlConnectionFail,
    NotFound,
    AlreadyExists,
    InvalidDomain,
    Unauthorized
}

impl Error {
    /// Get the error message. This is meant as a human-readable message to be shown on the client
    /// UI. In the future these strings may be translated, so clients should rely on `key()` to
    /// determine the error type.
    pub fn message(&self) -> &str {
        match *self {
            Error::Unknown => "Unknown error",
            Error::GravityError => "Gravity failed to form",
            Error::Custom(ref msg, _) => msg,
            Error::FtlConnectionFail => "Failed to connect to FTL",
            Error::NotFound => "Not found",
            Error::AlreadyExists => "Item already exists",
            Error::InvalidDomain => "Bad request",
            Error::Unauthorized => "Unauthorized"
        }
    }

    /// Get the error key. This should be used by clients to determine the error type instead of
    /// using the message because it will not change.
    pub fn key(&self) -> &str {
        match *self {
            Error::Unknown => "unknown",
            Error::GravityError => "gravity_error",
            Error::Custom(_, _) => "custom",
            Error::FtlConnectionFail => "ftl_connection_fail",
            Error::NotFound => "not_found",
            Error::AlreadyExists => "already_exists",
            Error::InvalidDomain => "invalid_domain",
            Error::Unauthorized => "unauthorized"
        }
    }

    /// Get the error HTTP status. This will be used when calling `reply_error`
    pub fn status(&self) -> Status {
        match *self {
            Error::Unknown => Status::InternalServerError,
            Error::GravityError => Status::InternalServerError,
            Error::Custom(_, status) => status,
            Error::FtlConnectionFail => Status::InternalServerError,
            Error::NotFound => Status::NotFound,
            Error::AlreadyExists => Status::Conflict,
            Error::InvalidDomain => Status::BadRequest,
            Error::Unauthorized => Status::Unauthorized
        }
    }

    pub fn as_outcome<S>(&self) -> request::Outcome<S, Self> {
        Outcome::Failure((self.status(), self.clone()))
    }
}

impl<T: Display> From<T> for Error {
    fn from(e: T) -> Self {
        // Cast to an Error by making a Error::Custom using the error's message
        Error::Custom(format!("{}", e), Status::InternalServerError)
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        // This allows us to automatically use `reply_error` when we return an Error in the API
        reply_error(self).unwrap().respond_to(request)
    }
}

/// This wraps another Responder and sets the HTTP status
#[derive(Debug)]
pub struct SetStatus<R>(R, Status);

impl<'r, R: Responder<'r>> Responder<'r> for SetStatus<R> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        // Set the status of the response
        Ok(Response::build_from(self.0.respond_to(request)?)
            .status(self.1)
            .finalize())
    }
}
