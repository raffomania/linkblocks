use garde::Report;

use crate::{
    form_errors::FormErrors,
    forms::users::{Credentials, Login},
};

#[derive(askama::Template, Default)]
#[template(path = "login.html")]
pub struct Template {
    errors: FormErrors,
    input: Login,
    oauth_google_configured: bool,
}

impl Template {
    pub fn new(errors: Report, input: Login, oauth_google_configured: bool) -> Self {
        Self {
            errors: errors.into(),
            input: Login {
                credentials: Credentials {
                    username: input.credentials.username,
                    // Never render the password we got from the user
                    password: String::new(),
                },
                ..input
            },
            oauth_google_configured,
        }
    }
}

#[derive(askama::Template, Default)]
#[template(path = "login_demo.html")]
pub struct DemoTemplate {}
