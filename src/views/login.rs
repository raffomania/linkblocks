use garde::Report;

use crate::{
    form_errors::FormErrors,
    forms::users::{Credentials, Login},
    oidc,
};

pub enum OidcInfo {
    NotConfigured,
    Configured { name: String },
}

impl Default for OidcInfo {
    fn default() -> Self {
        Self::NotConfigured
    }
}

impl From<oidc::State> for OidcInfo {
    fn from(value: oidc::State) -> Self {
        match value {
            oidc::State::NotConfigured => Self::NotConfigured,
            oidc::State::Configured(oidc::Config { client: _, name }) => Self::Configured { name },
        }
    }
}

#[derive(askama::Template, Default)]
#[template(path = "login.html")]
pub struct Template {
    errors: FormErrors,
    input: Login,
    oidc_info: OidcInfo,
}

impl Template {
    pub fn new(errors: Report, input: Login, oidc_state: oidc::State) -> Self {
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
            oidc_info: oidc_state.into(),
        }
    }
}

#[derive(askama::Template, Default)]
#[template(path = "login_demo.html")]
pub struct DemoTemplate {}
