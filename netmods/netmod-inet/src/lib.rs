//! A tcp overlay netmod to connect router across the internet

#[macro_use]
extern crate tracing;

mod peer;
mod routes;

pub struct Endpoint {
    routes: routes::Routes,
}
