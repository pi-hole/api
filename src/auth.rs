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

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_authenticated() {
        let mut data = Vec::new();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .ftl("getallqueries", data)
            .should_auth(true)
            .expect_json(json!({
                "data": [],
                "errors": []
            }))
            .test()
    }

    #[test]
    #[should_panic]
    fn test_unauthenticated() {
        let mut data = Vec::new();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .should_auth(false)
            .ftl("getallqueries", data)
            .expect_status(Status::Unauthorized)
            .expect_json(json!({
                "data": [],
                "errors": [{
                    "key": "unauthorized",
                    "message": "Unauthorized"
                }]
            }))
            .test()
    }
}