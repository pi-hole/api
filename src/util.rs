use serde::Serialize;
use rocket_contrib::{Json, Value};
use rocket::request::Request;
use rocket::response::{Response, Responder};

pub type Reply = CORS<Json<Value>>;

pub fn reply<D: Serialize>(data: D, errors: &[Error]) -> Reply {
    CORS(Json(json!({
        "data": data,
        "errors": errors.iter()
                        .map(|error| json!({
                            "key": error.key(),
                            "message": error.message()
                        }))
                        .collect::<Vec<Value>>()
    })))
}

pub fn reply_data<D: Serialize>(data: D) -> Reply {
    reply(data, &[])
}

pub fn reply_error(errors: Error) -> Reply {
    reply([0; 0], &[errors])
}

pub fn reply_success() -> Reply {
    reply(json!({
        "status": "success"
    }), &[])
}

pub enum Error {
    Unknown,
    AlreadyExists,
    DoesNotExist
}

impl Error {
    pub fn message(&self) -> &str {
        match *self {
            Error::Unknown => "Unknown error",
            Error::AlreadyExists => "Item already exists",
            Error::DoesNotExist => "Item does not exist"
        }
    }

    pub fn key(&self) -> &str {
        match *self {
            Error::Unknown => "unknown",
            Error::AlreadyExists => "already_exists",
            Error::DoesNotExist => "does_not_exist"
        }
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
