use rocket::request::{self, FromRequest, Request, State};
use rocket::Outcome;
use util;

/// When used as a request guard, requests must be authenticated
pub struct Auth;

/// Stores the API key in the server state
pub struct APIKey(String);

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = util::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let input_key = match request.headers().get_one("X-Pi-hole-Authenticate") {
            Some(key) => key,
            None => return util::Error::Unauthorized.as_outcome()
        };

        let api_key: State<APIKey> = match request.guard().succeeded() {
            Some(key) => key,
            None => return util::Error::Unknown.as_outcome()
        };

        if api_key.matches(input_key) {
            Outcome::Success(Auth {})
        } else {
            util::Error::Unauthorized.as_outcome()
        }
    }
}

impl APIKey {
    /// Create a new API key
    pub fn new(key: String) -> APIKey {
        APIKey(key)
    }

    /// Check if the key matches the server's key
    fn matches(&self, key: &str) -> bool {
        // TODO: harden this
        self.0 == key
    }
}
