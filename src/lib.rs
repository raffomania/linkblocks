#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::pedantic)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::redundant_closure_for_method_calls)]

mod authentication;
pub mod cli;
mod db;
mod extract;
mod form_errors;
mod forms;
mod oidc;
mod response_error;
mod routes;
pub mod server;
mod views;

mod htmf_response;
#[cfg(debug_assertions)]
mod insert_demo_data;
#[cfg(test)]
mod tests;
