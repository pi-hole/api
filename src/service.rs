// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Mockable Service Code
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

//! `Service` represents a mockable component in the application. During tests,
//! it can be replaced with a mock by inserting the mock into Rocket's state.
//!
//! A service is described by a trait which both the implementation and mock
//! implement. The code which uses the service references it as a trait object.
//!
//! Example:
//! ```
//! trait MyService {
//!     fn something(&self);
//! }
//!
//! // ...implementation and mock of the service...
//!
//! service!(MyServiceGuard, MyService, MyServiceImpl, MyServiceMock);
//!
//! #[get("/")]
//! fn handler(service: MyServiceGuard) {
//!     do_something(&*service);
//! }
//!
//! fn do_something(service: &dyn MyService) {
//!     service.something();
//! }
//! ```

use rocket::{
    request::{self, FromRequest},
    Outcome, Request
};
use std::ops::Deref;

#[cfg(test)]
use rocket::State;

/// Simplify creating a service. This will set up the service to work as
/// expected in tests and production. The first parameter is the identifier of
/// the request guard which will be created.
///
/// Example:
/// ```
/// service!(MyServiceGuard, MyService, MyServiceImpl, MyServiceMock);
///
/// // Now you can use the guard
/// #[get("/")]
/// fn handler(service: MyServiceGuard) {
///     // ...
/// }
/// ```
macro_rules! service {
    ($service_guard:ident, $service_trait:ident, $service_impl:ident, $service_mock:ident) => {
        #[cfg(not(test))]
        pub type $service_guard = $crate::service::Service<$service_trait, $service_impl>;

        #[cfg(test)]
        pub type $service_guard<'r> =
            $crate::service::Service<'r, $service_trait, $service_impl, $service_mock>;

        impl Deref for $service_impl {
            type Target = $service_trait;

            fn deref(&self) -> &Self::Target {
                self
            }
        }

        #[cfg(test)]
        impl Deref for $service_mock {
            type Target = $service_trait;

            fn deref(&self) -> &Self::Target {
                self
            }
        }
    };
}

/// The production version of Service. It is a simple wrapper around the
/// implementation.
#[cfg(not(test))]
pub struct Service<Trait, Impl>(Impl)
where
    Trait: ?Sized,
    Impl: Deref<Target = Trait>;

/// The test version of Service. It is either the implementation or the mock.
/// When it is being created as part of a request (FromRequest), it checks for
/// the mock in the state before trying to load the implementation.
#[cfg(test)]
pub enum Service<'r, Trait, Impl, Mock>
where
    Trait: ?Sized,
    Impl: Deref<Target = Trait>,
    Mock: Deref<Target = Trait> + Send + Sync + 'static
{
    Prod(Impl),
    Test(State<'r, Mock>)
}

// Dereference into the service trait
#[cfg(not(test))]
impl<Trait, Impl> Deref for Service<Trait, Impl>
where
    Trait: ?Sized,
    Impl: Deref<Target = Trait>
{
    type Target = Trait;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
impl<'r, Trait, Impl, Mock> Deref for Service<'r, Trait, Impl, Mock>
where
    Trait: ?Sized,
    Impl: Deref<Target = Trait>,
    Mock: Deref<Target = Trait> + Send + Sync + 'static
{
    type Target = Trait;

    fn deref(&self) -> &Self::Target {
        match self {
            Service::Prod(service) => service,
            Service::Test(state) => state.deref()
        }
    }
}

// Create the mock or impl service when a request comes in
#[cfg(not(test))]
impl<'a, 'r, Trait, Impl> FromRequest<'a, 'r> for Service<Trait, Impl>
where
    Trait: ?Sized,
    Impl: Deref<Target = Trait> + FromRequest<'a, 'r>
{
    type Error = Impl::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(Service(Impl::from_request(request)?))
    }
}

#[cfg(test)]
impl<'a, 'r, Trait, Impl, Mock> FromRequest<'a, 'r> for Service<'r, Trait, Impl, Mock>
where
    Trait: ?Sized,
    Impl: Deref<Target = Trait> + FromRequest<'a, 'r>,
    Mock: Deref<Target = Trait> + Send + Sync + 'static
{
    type Error = Impl::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if let Some(mock) = request.guard::<State<Mock>>().succeeded() {
            return Outcome::Success(Service::Test(mock));
        }

        Outcome::Success(Service::Prod(Impl::from_request(request)?))
    }
}
