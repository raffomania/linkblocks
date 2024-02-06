mod authentication;
pub mod cli;
mod db;
mod form_errors;
mod response_error;
mod routes;
mod schemas;
pub mod server;
mod views;

pub mod insert_demo_data;
#[cfg(test)]
mod tests;
