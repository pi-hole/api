// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// General API Utilities
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use failure::{Backtrace, Context, Fail};
use rocket::{
    http::Status,
    request,
    response::{self, Responder, Response},
    Outcome, Request
};
use rocket_contrib::json::JsonValue;
use serde::Serialize;
use shmem;
use std::{
    env,
    fmt::{self, Display},
    sync::Arc
};

/// Type alias for the most common return type of the API methods
pub type Reply = Result<SetStatus<JsonValue>, Error>;

/// The most general reply builder. It takes in data/errors and status to
/// construct the JSON reply.
pub fn reply<D: Serialize>(data: Result<D, Error>, status: Status) -> Reply {
    let json_data = match data {
        Ok(d) => json!(d),
        Err(e) => {
            // Only print out the error if it's not a common error
            match e.kind() {
                ErrorKind::Unauthorized | ErrorKind::NotFound => (),
                _ => e.print_stacktrace()
            }

            // Get the extra error data, or null if there is none
            let data = e.data().unwrap_or_default();

            json!({
                "error": {
                    "key": e.key(),
                    "message": format!("{}", e),
                    "data": data
                }
            })
        }
    };

    Ok(SetStatus(json_data, status))
}

/// Create a reply from a Result of serializable data or an error. If the Result
/// is Ok, [`reply_data`] will be used. If the Result is Err, [`reply_error`]
/// will be used.
///
/// [`reply_data`]: fn.reply_data.html
/// [`reply_error`]: fn.reply_error.html
pub fn reply_result<D: Serialize>(data: Result<D, Error>) -> Reply {
    match data {
        Ok(data) => reply_data(data),
        Err(error) => reply_error(error)
    }
}

/// Create a reply from some serializable data. The reply will have a status
/// code of 200.
pub fn reply_data<D: Serialize>(data: D) -> Reply {
    reply(Ok(data), Status::Ok)
}

/// Create a reply with an error. The status will taken from `error.status()`.
pub fn reply_error<E: Into<Error>>(error: E) -> Reply {
    let error = error.into();
    let status = error.status();
    reply::<()>(Err(error), status)
}

/// Create a reply with a successful status. There are no errors and the status
/// is 200.
pub fn reply_success() -> Reply {
    reply(Ok(json!({ "status": "success" })), Status::Ok)
}

/// Wraps `ErrorKind` to provide context via `Context`.
///
/// See https://boats.gitlab.io/failure/error-errorkind.html
#[derive(Debug, Clone)]
pub struct Error {
    inner: Arc<Context<ErrorKind>>
}

/// The `ErrorKind` enum represents all the possible errors that the API can
/// return.
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Unknown error")]
    Unknown,
    #[fail(display = "Failed to create the blocklist")]
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
    ConfigParsingError,
    #[fail(display = "Invalid setting value")]
    InvalidSettingValue,
    #[fail(display = "Failed to restart the DNS server")]
    RestartDnsError,
    #[fail(display = "Failed to reload the DNS server")]
    ReloadDnsError,
    #[fail(display = "Error generating the dnsmasq config")]
    DnsmasqConfigWrite,
    /// `shmem::Error` does not implement `std::error::Error`, so we can not use
    /// `.context()` on a `Result<T, shmem::Error>`. It also does not implement
    /// `Eq` or `PartialEq`, so the best we can do is have the error message
    /// stored here.
    #[fail(display = "Failed to open shared memory: {}", _0)]
    SharedMemoryOpen(String),
    #[fail(display = "Failed to read from shared memory")]
    SharedMemoryRead,
    #[fail(display = "Failed to lock shared memory")]
    SharedMemoryLock,
    #[fail(
        display = "Incompatible version of shared memory. Found {}, expected {}",
        _0, _1
    )]
    SharedMemoryVersion(usize, usize),
    #[fail(display = "Error while interacting with the FTL database")]
    FtlDatabase,
    #[fail(display = "Error while interacting with the Gravity database")]
    GravityDatabase,
    #[fail(display = "Missing key")]
    LdapMissingKey,
    #[fail(display = "Missing username")]
    LdapMissingUsername,
    #[fail(display = "Bind error")]
    LdapBindError,
    #[fail(display = "Unauthorized")]
    LdapUnauthorized,
    #[fail(display = "Connection error")]
    LdapConnectError
}

impl Error {
    pub fn print_stacktrace(&self) {
        eprintln!("Error: {}", self);

        // Only print the backtrace if requested, to avoid a gap between error and
        // causes
        let backtrace_enabled = env::var("RUST_BACKTRACE").is_ok();
        if backtrace_enabled {
            if let Some(backtrace) = self.backtrace() {
                eprintln!("{}", backtrace);
            }
        }

        // Print out each cause
        for (i, cause) in Fail::iter_causes(self).enumerate() {
            eprintln!("Cause #{}: {}", i + 1, cause);

            if backtrace_enabled {
                if let Some(backtrace) = cause.backtrace() {
                    eprintln!("{}", backtrace);
                }
            }
        }
    }

    /// Get the wrapped [`ErrorKind`]
    ///
    /// [`ErrorKind`]: enum.ErrorKind.html
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
    }

    /// Get extra data about the error from the [`ErrorKind`]
    ///
    /// [`ErrorKind`]: enum.ErrorKind.html
    fn data(&self) -> Option<JsonValue> {
        self.inner.get_context().data()
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
    /// Get the error key. This should be used by clients to determine the
    /// error type instead of using the message because it will not change.
    pub fn key(&self) -> &'static str {
        match self {
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
            ErrorKind::ConfigParsingError => "config_parsing_error",
            ErrorKind::InvalidSettingValue => "invalid_setting_value",
            ErrorKind::RestartDnsError => "restart_dns_error",
            ErrorKind::ReloadDnsError => "reload_dns_error",
            ErrorKind::DnsmasqConfigWrite => "dnsmasq_config_write",
            ErrorKind::SharedMemoryOpen(_) => "shared_memory_open",
            ErrorKind::SharedMemoryRead => "shared_memory_read",
            ErrorKind::SharedMemoryLock => "shared_memory_lock",
            ErrorKind::SharedMemoryVersion(_, _) => "shared_memory_version",
            ErrorKind::FtlDatabase => "ftl_database",
            ErrorKind::GravityDatabase => "gravity_database",
            ErrorKind::LdapMissingKey => "ldap_missing_key",
            ErrorKind::LdapMissingUsername => "ldap_missing_username",
            ErrorKind::LdapConnectError => "ldap_connection_error",
            ErrorKind::LdapBindError => "ldap_bind_error",
            ErrorKind::LdapUnauthorized => "ldap_unauthorized"
        }
    }

    /// Get the error HTTP status. This will be used when calling `reply_error`
    pub fn status(&self) -> Status {
        match self {
            ErrorKind::NotFound => Status::NotFound,
            ErrorKind::AlreadyExists => Status::Conflict,
            ErrorKind::InvalidDomain
            | ErrorKind::BadRequest
            | ErrorKind::InvalidSettingValue
            | ErrorKind::LdapMissingUsername
            | ErrorKind::LdapMissingKey => Status::BadRequest,
            ErrorKind::Unauthorized | ErrorKind::LdapUnauthorized => Status::Unauthorized,
            ErrorKind::Unknown
            | ErrorKind::GravityError
            | ErrorKind::FtlConnectionFail
            | ErrorKind::FtlReadError
            | ErrorKind::FtlEomError
            | ErrorKind::FileRead(_)
            | ErrorKind::FileWrite(_)
            | ErrorKind::ConfigParsingError
            | ErrorKind::RestartDnsError
            | ErrorKind::ReloadDnsError
            | ErrorKind::DnsmasqConfigWrite
            | ErrorKind::SharedMemoryOpen(_)
            | ErrorKind::SharedMemoryRead
            | ErrorKind::SharedMemoryLock
            | ErrorKind::SharedMemoryVersion(_, _)
            | ErrorKind::FtlDatabase
            | ErrorKind::GravityDatabase
            | ErrorKind::LdapBindError
            | ErrorKind::LdapConnectError => Status::InternalServerError
        }
    }

    /// Get extra data about the error, to be used in the JSON error object
    fn data(&self) -> Option<JsonValue> {
        match self {
            ErrorKind::FileRead(file) => Some(json!({ "file": file })),
            ErrorKind::FileWrite(file) => Some(json!({ "file": file })),
            _ => None
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
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
        Error {
            inner: Arc::new(Context::new(kind))
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error {
            inner: Arc::new(inner)
        }
    }
}

impl From<shmem::Error> for Error {
    /// Converts `shmem::Error` into an `Error` of kind
    /// [`ErrorKind::SharedMemoryOpen`]. See the comment on
    /// [`ErrorKind::SharedMemoryOpen`] for more information.
    ///
    /// [`ErrorKind::SharedMemoryOpen`]:
    /// enum.ErrorKind.html#variant.SharedMemoryOpen
    fn from(e: shmem::Error) -> Self {
        Error::from(ErrorKind::SharedMemoryOpen(format!("{:?}", e)))
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        // This allows us to automatically use `reply_error` when we return an Error in
        // the API
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
