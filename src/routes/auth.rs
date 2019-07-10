// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Authentication Functions And Routes
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::util::{reply_success, Error, ErrorKind, Reply};
use rocket::{
    http::{Cookie, Cookies},
    request::{self, FromRequest, Request, State},
    Outcome
};
use std::sync::atomic::{AtomicUsize, Ordering};

const USER_ATTR: &str = "user_id";
const AUTH_HEADER: &str = "X-Pi-hole-Authenticate";

/// When used as a request guard, requests must be authenticated
pub struct User {
    pub id: usize
}

/// Stores the API key in the server state
pub struct AuthData {
    key: Option<String>,
    next_id: AtomicUsize
}

impl User {
    /// Try to get the user ID from cookies
    fn get_from_cookie(cookies: &mut Cookies) -> Option<Self> {
        cookies
            .get_private(USER_ATTR)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User { id })
    }

    /// Create a new user and store the ID in a cookie
    fn create_and_store_user(request: &Request, auth_data: &AuthData) -> User {
        let user = auth_data.create_user();

        // Set a new encrypted cookie with the user's ID
        request.cookies().add_private(
            Cookie::build(USER_ATTR, user.id.to_string())
                // Allow the web interface to read the cookie
                .http_only(false)
                .finish()
        );

        user
    }

    /// Log the user out by removing the cookie
    fn logout(&self, mut cookies: Cookies) {
        cookies.remove_private(Cookie::named(USER_ATTR));
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        // Check if the user has already authenticated and has a valid cookie
        if let Some(user) = User::get_from_cookie(&mut request.cookies()) {
            return Outcome::Success(user);
        }

        // Load the auth data
        let auth_data: State<AuthData> = match request.guard().succeeded() {
            Some(auth_data) => auth_data,
            None => return Error::from(ErrorKind::Unknown).into_outcome()
        };

        // Check if a key is required for authentication
        if !auth_data.key_required() {
            return Outcome::Success(User::create_and_store_user(request, &auth_data));
        }

        // Check the user's key, if provided
        if let Some(key) = request.headers().get_one(AUTH_HEADER) {
            if auth_data.key_matches(key) {
                // The key matches, so create and store a new user and cookie
                Outcome::Success(Self::create_and_store_user(request, &auth_data))
            } else {
                // The key does not match
                Error::from(ErrorKind::Unauthorized).into_outcome()
            }
        } else {
            // A key is required but not provided
            Error::from(ErrorKind::Unauthorized).into_outcome()
        }
    }
}

impl AuthData {
    /// Create a new API key
    pub fn new(key: Option<String>) -> AuthData {
        AuthData {
            key,
            next_id: AtomicUsize::new(1)
        }
    }

    /// Check if the key matches the server's key
    fn key_matches(&self, key: &str) -> bool {
        self.key
            .as_ref()
            // If a password is required, check that the given one matches
            .map(|api_key| api_key == key)
            // If no password is required, authenticate the user
            .unwrap_or(true)
    }

    /// Check if a key is required to authenticate
    fn key_required(&self) -> bool {
        self.key.is_some()
    }

    /// Create a new user and increment `next_id`
    fn create_user(&self) -> User {
        User {
            id: self.next_id.fetch_add(1, Ordering::Relaxed)
        }
    }
}

/// Provides an endpoint to authenticate or check if already authenticated
#[get("/auth")]
pub fn check(_user: User) -> Reply {
    reply_success()
}

/// Clears the user's authentication
#[delete("/auth")]
pub fn logout(user: User, cookies: Cookies) -> Reply {
    user.logout(cookies);
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::testing::TestBuilder;
    use rocket::http::{Header, Status};
    use serde_json::Value;

    /// Providing the correct authentication should authorize the request
    #[test]
    fn authenticated() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(true)
            .expect_json(json!({
                "status": "success"
            }))
            .test()
    }

    /// Providing no authorization should not authorize the request
    #[test]
    fn unauthenticated() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(false)
            .expect_status(Status::Unauthorized)
            .expect_json(json!({
                "error": {
                    "key": "unauthorized",
                    "message": "Unauthorized",
                    "data": Value::Null
                }
            }))
            .test()
    }

    /// Providing incorrect authorization should not authorize the request
    #[test]
    fn wrong_password() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(false)
            .header(Header::new(
                "X-Pi-hole-Authenticate",
                "obviously_not_correct"
            ))
            .expect_status(Status::Unauthorized)
            .expect_json(json!({
                "error": {
                    "key": "unauthorized",
                    "message": "Unauthorized",
                    "data": Value::Null
                }
            }))
            .test();
    }

    /// If no password is set for the API, an unauthenticated auth request is
    /// authorized
    #[test]
    fn no_password_required() {
        TestBuilder::new()
            .endpoint("/admin/api/auth")
            .should_auth(false)
            .auth_required(false)
            .expect_json(json!({
                "status": "success"
            }))
            .test();
    }
}
