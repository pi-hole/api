use rocket::request::{self, FromRequest, Request, State};
use rocket::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::http::Cookies;
use std::sync::atomic::{Ordering, AtomicUsize};
use util;

/// When used as a request guard, requests must be authenticated
pub struct User {
    id: usize
}

/// Stores the API key in the server state
pub struct AuthData {
    key: String,
    next_id: AtomicUsize
}

impl User {
    fn authenticate(request: &Request, input_key: &str) -> request::Outcome<Self, util::Error> {
        let auth_data: State<AuthData> = match request.guard().succeeded() {
            Some(auth_data) => auth_data,
            None => return util::Error::Unknown.as_outcome()
        };

        if auth_data.key_matches(input_key) {
            Outcome::Success(auth_data.add_user())
        } else {
            util::Error::Unauthorized.as_outcome()
        }
    }

    fn check_cookies(mut cookies: Cookies) -> request::Outcome<Self, util::Error> {
        cookies
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User { id })
            .into_outcome((util::Error::Unauthorized.status(), util::Error::Unauthorized))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = util::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get_one("X-Pi-hole-Authenticate") {
            // Try to authenticate, and if that fails check cookies
            Some(key) => {
                let auth_result = User::authenticate(request, key);

                if auth_result.is_success() {
                    auth_result
                } else {
                    User::check_cookies(request.cookies())
                }
            },
            // No attempt to authenticate, so check cookies
            None => User::check_cookies(request.cookies())
        }
    }
}

impl AuthData {
    /// Create a new API key
    pub fn new(key: String) -> AuthData {
        AuthData { key, next_id: AtomicUsize::new(1) }
    }

    /// Check if the key matches the server's key
    fn key_matches(&self, key: &str) -> bool {
        // TODO: harden this
        self.key == key
    }

    /// Create a new user and increment `next_id`
    fn add_user(&self) -> User {
        User { id: self.next_id.fetch_add(1, Ordering::Relaxed) }
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