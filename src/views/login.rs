use askama::Template;
use garde::Report;

use crate::{form_errors::FormErrors, forms::users::Credentials};

#[derive(Template, Default)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    errors: FormErrors,
    input: Credentials,
}

impl LoginTemplate {
    pub fn new(errors: Report, input: Credentials) -> Self {
        Self {
            errors: errors.into(),
            input: Credentials {
                username: input.username,
                // Never render the password we got from the user
                password: String::new(),
            },
        }
    }
}
