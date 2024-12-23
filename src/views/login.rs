use garde::Report;
#[allow(clippy::wildcard_imports)]
use htmf::prelude::*;

use crate::{
    form_errors::FormErrors,
    forms::users::{Credentials, Login},
    oidc,
};

use super::{base_document::base_document, forms::form_errors};

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

#[derive(Default)]
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

pub fn login(template: &Template) -> Element {
    base_document(
        div(class(
            "flex flex-col justify-center max-w-md min-h-full px-4 mx-auto",
        ))
        .with([login_form(template), oidc_button(&template.oidc_info)]),
    )
}

fn login_form(template: &Template) -> Element {
    form([
        action("/login"),
        method("post"),
        attr("hx-boost", "true"),
        attr("hx-disabled-elt", "button"),
        class("flex flex-col w-full"),
    ])
    .with([
        h1(class("text-2xl font-bold tracking-tight text-center")).with("Sign in to your account"),
        username_field(&template.errors, &template.input.credentials.username),
        password_field(&template.errors),
        template
            .input
            .previous_uri
            .as_ref()
            .map(|previous_uri| input([type_("hidden"), name("previous_uri"), value(previous_uri)]))
            .into(),
        submit_button(),
        form_errors(&template.errors, "root"),
    ])
}

fn username_field(errors: &FormErrors, val: &str) -> Element {
    fragment().with([
        label([
            class("mt-10 text-neutral-400"),
            for_("credentials[username]"),
        ])
        .with("Username"),
        form_errors(errors, "credentials.username"),
        input([
            type_("text"),
            name("credentials[username]"),
            class("rounded py-1.5 px-3 mt-2 bg-neutral-900"),
            value(val),
            required("true"),
        ]),
    ])
}

fn password_field(errors: &FormErrors) -> Element {
    fragment().with([
        label([
            class("mt-4 text-neutral-400"),
            for_("credentials[password]"),
        ])
        .with("Password"),
        form_errors(errors, "credentials.password"),
        input([
            type_("password"),
            name("credentials[password]"),
            class("rounded py-1.5 px-3 mt-2 bg-neutral-900"),
            required("true"),
        ]),
    ])
}

fn submit_button() -> Element {
    button([
        type_("submit"),
        class(
            "leading-6 bg-neutral-300 mt-5 font-semibold rounded py-1.5 flex items-center \
             justify-center disabled:bg-neutral-500 text-neutral-900",
        ),
    ])
    .with([
        span(class("inline-block w-0 h-4")).with(span(class(
            "block w-4 h-4 -ml-6 border-2 rounded-full border-neutral-900 animate-spin \
             border-t-transparent htmx-indicator",
        ))),
        text("Sign in"),
    ])
}

fn oidc_button(oidc_info: &OidcInfo) -> Element {
    if let OidcInfo::Configured { name } = oidc_info {
        fragment().with([
            hr(class("my-5 border-neutral-700")),
            a([
                class(
                    "leading-6 border border-neutral-500 font-semibold rounded py-1.5 flex \
                     items-center justify-center",
                ),
                href("/login_oidc"),
            ])
            .with(format!("Sign in with {name}")),
        ])
    } else {
        nothing()
    }
}

#[derive(askama::Template, Default)]
#[template(path = "login_demo.html")]
pub struct DemoTemplate {}
