use htmf::prelude_inline::*;

use super::layout;

struct ProfileData {
    username: String,
}

pub fn view(layout: &layout::Template, data: &ProfileData) -> Element {
    let content = fragment([header(class("mx-4 mt-3 mb-4"), [h1((), &data.username)])]);

    super::layout::layout(content, layout)
}
