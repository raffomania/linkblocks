use htmf::prelude_inline::*;

use crate::{db, views::layout};

pub struct Data {
    pub layout: layout::Template,
    pub ap_user: db::ApUser,
    pub lists: Vec<db::lists::List>,
}

// TODO should this page exist for remote ApUsers as well?
pub fn view(
    Data {
        layout,
        ap_user,
        lists,
    }: &Data,
) -> Element {
    let children = fragment(
        header(
            [class("px-4 pt-3 mb-2")],
            [
                h1([class("text-xl font-bold")], [&ap_user.username]),
                ap_user.bio.as_ref().map_or(nothing(), |bio| p((), bio)),
            ],
        )
        .with(view_lists(lists)),
    );

    layout::layout(children, layout)
}

fn view_lists(lists: &[db::lists::List]) -> Element {
    section(
        [],
        lists
            .iter()
            .map(|list| {
                section(
                    class(
                        "flex flex-wrap items-end gap-2 px-4 pt-4 pb-4 border-t border-neutral-700",
                    ),
                    [a(
                        [
                            class(
                                "block overflow-hidden font-semibold leading-8 \
                                 hover:text-fuchsia-300 text-ellipsis whitespace-nowrap",
                            ),
                            href(list.path()),
                        ],
                        &list.title,
                    )],
                )
            })
            .collect::<Vec<_>>(),
    )
}
