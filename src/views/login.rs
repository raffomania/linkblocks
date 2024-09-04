use garde::Report;

use crate::{
    form_errors::FormErrors,
    forms::users::{Credentials, Login},
    server::{OauthConfig, OauthState},
};

pub enum OauthInfo {
    NotConfigured,
    Configured { name: String },
}

impl Default for OauthInfo {
    fn default() -> Self {
        Self::NotConfigured
    }
}

impl From<OauthState> for OauthInfo {
    fn from(value: OauthState) -> Self {
        match value {
            OauthState::NotConfigured => Self::NotConfigured,
            OauthState::Configured(OauthConfig { client: _, name }) => Self::Configured { name },
        }
    }
}

#[derive(askama::Template, Default)]
#[template(path = "login.html")]
pub struct Template {
    errors: FormErrors,
    input: Login,
    oauth_info: OauthInfo,
}

impl Template {
    pub fn new(errors: Report, input: Login, oauth_state: OauthState) -> Self {
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
            oauth_info: oauth_state.into(),
        }
    }
}

#[derive(askama::Template, Default)]
#[template(path = "login_demo.html")]
pub struct DemoTemplate {}
