use crate::{form_errors::FormErrors, forms::users::OidcSelectUsername};

#[derive(askama::Template, Default)]
#[template(path = "oidc_select_username.html")]
pub struct Template {
    pub errors: FormErrors,
    pub input: OidcSelectUsername,
}
