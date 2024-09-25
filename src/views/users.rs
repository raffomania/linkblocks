use askama::Template;
#[allow(clippy::wildcard_imports)]
use htmf::declare::*;
use htmf::Element;

use super::layout::{self, layout};

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub layout: layout::Template,
    pub base_url: String,
}

pub fn profile(template: &ProfileTemplate) -> Element {
    layout(
        vec![
            header().class("px-4 pt-3 mb-2").with([h1()
                .class("text-xl font-bold")
                .with([text("Install Bookmarklet")])]),
            section()
                .class("p-4")
                .with([bookmarklet_help(), bookmarklet()]),
        ],
        &template.layout,
    )
}

fn bookmarklet_help() -> Element<'static> {
    fragment([
        p().class("mb-2").with([text(
            "Click the bookmarklet on any website to add it as a bookmark in
      linkblocks!",
        )]),
        p().with([text(
            "To install, drag the following link to your bookmarks / favorites toolbar:",
        )]),
    ])
}

fn bookmarklet() -> Element<'static> {
    // window.open(
    //   "{{ base_url }}/bookmarks/create?url="
    //   +encodeURIComponent(window.location.href)
    //   +"&title="
    //   +encodeURIComponent(document.title)
    // )
    a().class("block my-2 font-bold text-orange-200")
        .href(
            "javascript:(function()%7Bwindow.open(%0A%20%20%22{{ base_url \
             }}%2Fbookmarks%2Fcreate%3Furl%3D%22%0A%20%20%2BencodeURIComponent(window.location.\
             href)%0A%20%20%2B%22%26title%3D%22%0A%20%20%2BencodeURIComponent(document.title)%0A)%\
             7D)()",
        )
        .with([text("Add to linkblocks")])
}
