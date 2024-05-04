mod authentication;
pub mod cli;
mod db;
mod extract;
mod form_errors;
mod forms;
mod import;
mod response_error;
mod routes;
pub mod server;
mod views;

pub mod insert_demo_data;
#[cfg(test)]
mod tests;
