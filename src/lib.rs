#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::redundant_closure_for_method_calls)]

mod authentication;
pub mod cli;
mod db;
mod extract;
mod form_errors;
mod forms;
mod response_error;
mod routes;
pub mod server;
mod views;

pub mod insert_demo_data;
#[cfg(test)]
mod tests;
