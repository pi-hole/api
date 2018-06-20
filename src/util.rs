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
use std::fmt::{self, Display};
use failure::{Context, Fail, Backtrace};

/// Type alias for the most common return type of the API methods
pub type Reply = Result<SetStatus<Json<Value>>, Error>;

/// The most general reply builder. It takes in data/errors and status to construct the JSON reply.
pub fn reply<D: Serialize>(data: ReplyType<D>, status: Status) -> Reply {
    let json_data = match data {
        ReplyType::Data(d) => json!(d),
        ReplyType::Error(e) => json!({
            "error": {
                "key": e.key(),
                "message": format!("{}", e)
            }
        })
    };

    Ok(SetStatus(Json(json_data), status))
}

/// Create a reply from some serializable data. The reply will contain no errors and will have a
/// status code of 200.
pub fn reply_data<D: Serialize>(data: D) -> Reply {
    reply(ReplyType::Data(data), Status::Ok)
}

/// Create a reply with an error. The data will be an empty array and the status will taken from
/// `error.status()`.
pub fn reply_error<E: Into<Error>>(error: E) -> Reply {
    let error = error.into();
    let status = error.status();
    reply::<()>(ReplyType::Error(error), status)
}

/// Create a reply with a successful status. There are no errors and the status is 200.
pub fn reply_success() -> Reply {
    reply(ReplyType::Data(json!({ "status": "success" })), Status::Ok)
}

pub enum ReplyType<D: Serialize> {
    Data(D), Error(Error)
}

/// Wraps `ErrorKind` to provide context via `Context`.
///
/// See https://boats.gitlab.io/failure/error-errorkind.html
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>
}

/// The `ErrorKind` enum represents all the possible errors that the API can return.
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Unknown error")]
    Unknown,
    #[fail(display = "Gravity failed to form")]
    GravityError,
    #[fail(display = "Failed to connect to FTL")]
    FtlConnectionFail,
    #[fail(display = "Error reading from FTL")]
    FtlReadError,
    #[fail(display = "Read unexpected EOM from FTL")]
    FtlEomError,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Item already exists")]
    AlreadyExists,
    #[fail(display = "Invalid domain")]
    InvalidDomain,
    #[fail(display = "Bad request")]
    BadRequest,
    #[fail(display = "Unauthorized")]
    Unauthorized,
    #[fail(display = "Error reading from {}", _0)]
    FileRead(String),
    #[fail(display = "Error writing to {}", _0)]
    FileWrite(String),
    #[fail(display = "Error parsing the config")]
    ConfigParsingError
}

impl Error {
    /// Get the wrapped [`ErrorKind`]
    ///
    /// [`ErrorKind`]: enum.ErrorKind.html
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
    }

    /// See [`ErrorKind::key`]
    ///
    /// [`ErrorKind::key`]: enum.ErrorKind.html#method.key
    pub fn key(&self) -> &'static str {
        self.kind().key()
    }

    /// See [`ErrorKind::status`]
    ///
    /// [`ErrorKind::status`]: enum.ErrorKind.html#method.status
    pub fn status(&self) -> Status {
        self.kind().status()
    }

    pub fn into_outcome<S>(self) -> request::Outcome<S, Self> {
        Outcome::Failure((self.status(), self))
    }
}

impl ErrorKind {
    /// Get the error key. This should be used by clients to determine the error type instead of
    /// using the message because it will not change.
    pub fn key(&self) -> &'static str {
        match *self {
            ErrorKind::Unknown => "unknown",
            ErrorKind::GravityError => "gravity_error",
            ErrorKind::FtlConnectionFail => "ftl_connection_fail",
            ErrorKind::FtlReadError => "ftl_read_error",
            ErrorKind::FtlEomError => "ftl_eom_error",
            ErrorKind::NotFound => "not_found",
            ErrorKind::AlreadyExists => "already_exists",
            ErrorKind::InvalidDomain => "invalid_domain",
            ErrorKind::BadRequest => "bad_request",
            ErrorKind::Unauthorized => "unauthorized",
            ErrorKind::FileRead(_) => "file_read",
            ErrorKind::FileWrite(_) => "file_write",
            ErrorKind::ConfigParsingError => "config_parsing_error"
        }
    }

    /// Get the error HTTP status. This will be used when calling `reply_error`
    pub fn status(&self) -> Status {
        match *self {
            ErrorKind::Unknown => Status::InternalServerError,
            ErrorKind::GravityError => Status::InternalServerError,
            ErrorKind::FtlConnectionFail => Status::InternalServerError,
            ErrorKind::FtlReadError => Status::InternalServerError,
            ErrorKind::FtlEomError => Status::InternalServerError,
            ErrorKind::NotFound => Status::NotFound,
            ErrorKind::AlreadyExists => Status::Conflict,
            ErrorKind::InvalidDomain => Status::BadRequest,
            ErrorKind::BadRequest => Status::BadRequest,
            ErrorKind::Unauthorized => Status::Unauthorized,
            ErrorKind::FileRead(_) => Status::InternalServerError,
            ErrorKind::FileWrite(_) => Status::InternalServerError,
            ErrorKind::ConfigParsingError => Status::InternalServerError
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
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
