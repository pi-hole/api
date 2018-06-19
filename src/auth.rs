use rocket::request::{self, FromRequest, Request, State};
use rocket::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::http::{Cookie, Cookies};
use std::sync::atomic::{Ordering, AtomicUsize};
use util;

const USER_ATTR: &str = "user_id";
const AUTH_HEADER: &str = "X-Pi-hole-Authenticate";

/// When used as a request guard, requests must be authenticated
pub struct User {
    pub id: usize
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
            None => return util::ErrorKind::Unknown.into().as_outcome()
        };

        if auth_data.key_matches(input_key) {
            let user = auth_data.create_user();
            request.cookies().add_private(Cookie::new(USER_ATTR, user.id.to_string()));

            Outcome::Success(user)
        } else {
            util::ErrorKind::Unauthorized.into().as_outcome()
        }
    }

    fn check_cookies(mut cookies: Cookies) -> request::Outcome<Self, util::Error> {
        cookies
            .get_private(USER_ATTR)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User { id })
            .ok_or_else(|| util::ErrorKind::Unauthorized.into().as_outcome())
    }

    fn logout(&self, mut cookies: Cookies) {
        cookies.remove_private(Cookie::named(USER_ATTR));
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = util::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get_one(AUTH_HEADER) {
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
        self.key == key
    }

    /// Create a new user and increment `next_id`
    fn create_user(&self) -> User {
        User { id: self.next_id.fetch_add(1, Ordering::Relaxed) }
    }
}

/// Provides an endpoint to authenticate or check if already authenticated
#[get("/auth")]
pub fn check(_user: User) -> util::Reply {
    util::reply_success()
}

/// Clears the user's authentication
#[delete("/auth")]
pub fn logout(user: User, cookies: Cookies) -> util::Reply {
    user.logout(cookies);
    util::reply_success()
}

#[cfg(test)]
mod test {
    use rocket::http::{Status, Header};
    use testing::TestBuilder;

    #[test]
    fn test_authenticated() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(true)
            .expect_json(json!({
                "status": "success"
            }))
            .test()
    }

    #[test]
    fn test_unauthenticated() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(false)
            .expect_status(Status::Unauthorized)
            .expect_json(json!({
                "error": {
                    "key": "unauthorized",
                    "message": "Unauthorized"
                }
            }))
            .test()
    }

    #[test]
    fn test_wrong_password() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(false)
            .header(Header::new("X-Pi-hole-Authenticate", "obviously_not_correct"))
            .expect_status(Status::Unauthorized)
            .expect_json(json!({
                "error": {
                    "key": "unauthorized",
                    "message": "Unauthorized"
                }
            }))
            .test();
    }
}