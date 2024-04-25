use garde::Validate;
use openidconnect::{AuthorizationCode, CsrfToken};
use serde::{Deserialize, Serialize};

#[derive(Validate)]
pub struct CreateUser {
    #[garde(alphanumeric, ascii, length(min = 3, max = 50))]
    pub username: String,
    #[garde(length(min = 10, max = 100))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Default)]
pub struct Login {
    #[garde(skip)]
    pub previous_uri: Option<String>,
    #[garde(dive)]
    pub credentials: Credentials,
}

#[derive(Serialize, Deserialize, Validate, Debug, Default)]
pub struct Credentials {
    #[garde(alphanumeric, ascii, length(min = 3, max = 50))]
    pub username: String,
    #[garde(length(min = 10, max = 100))]
    pub password: String,
}

#[derive(Deserialize)]
pub struct OidcLoginQuery {
    pub code: AuthorizationCode,
    pub state: CsrfToken,
}

#[derive(Serialize, Deserialize, Validate, Debug, Default)]
pub struct CreateOidcUser {
    #[garde(length(min = 10, max = 100))]
    pub oidc_id: String,
    #[garde(length(min = 10, max = 100))]
    pub email: String,
}
