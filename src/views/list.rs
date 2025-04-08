use htmf::prelude::*;

use super::{content, layout};
use crate::db::{self, LinkWithContent};

pub struct Data {
    pub layout: layout::Template,
    pub links: Vec<db::LinkWithContent>,
    pub list: db::List,
    pub metadata: db::lists::Metadata,
}

pub fn view(
    data @ Data {
        layout,
        links,
        list,
        metadata,
    }: &Data,
) -> Element {
    layout::layout(
        fragment()
            .with([header(class("px-4 pt-3 mb-4"))
                .with([
                    div(class("flex items-center justify-between"))
                        .with([h1(class("text-xl font-bold")).with(&list.title)]),
                    div(class("flex flex-wrap text-sm gap-x-1 text-neutral-400")).with([
                        p([]).with(format!("by {}", metadata.user_description)),
                        text("∙"),
                        p([]).with(format!("{} bookmarks", metadata.linked_bookmark_count)),
                        text("∙"),
                        p([]).with(format!("{} lists", metadata.linked_list_count)),
                        text("∙"),
                        p(id("private_indicator")).with(if list.private {
                            "private"
                        } else {
                            "public"
                        }),
                    ]),
                ])
                .with(list.content.as_ref().and_then(|content| {
                    (!content.is_empty()).then_some(p(class("max-w-2xl mt-2")).with(content))
                }))])
            .with(layout.authed_info.as_ref().and_then(|authed_info| {
                (authed_info.user_id == list.user_id).then(|| edit_buttons(data))
            }))
            .with(
                links
                    .iter()
                    .map(|link| list_item(link, data))
                    .collect::<Vec<_>>(),
            ),
        layout,
    )
}

fn edit_buttons(Data { list, .. }: &Data) -> Element {
    section(class("flex flex-wrap m-4 gap-x-4 gap-y-2")).with([
        a([
            class("block px-4 py-1 border rounded hover:bg-neutral-700 border-neutral-700 w-max"),
            href(format!("/links/create?dest_id={}", list.id)),
        ])
        .with("Add to other list"),
        form([
            action(format!("/lists/{}/edit_private", list.id)),
            attr("hx-post", format!("/lists/{}/edit_private", list.id)),
            attr("hx-select", "#edit_private"),
            attr("hx-select-oob", "#private_indicator"),
            attr("hx-target", "this"),
            id("edit_private"),
            method("post"),
        ])
        .with([button([
            class("block px-4 py-1 border rounded hover:bg-neutral-700 border-neutral-700 w-max"),
            name("private"),
            type_("submit"),
            value(if list.private { "false" } else { "true" }),
        ])
        .with(if list.private {
            "Make public"
        } else {
            "Make private"
        })]),
        form([
            action(format!("/lists/{}/edit_pinned", list.id)),
            id("edit_pinned"),
            method("post"),
        ])
        .with([button([
            class("block px-4 py-1 border rounded hover:bg-neutral-700 border-neutral-700 w-max"),
            name("pinned"),
            type_("submit"),
            value(if list.pinned { "false" } else { "true" }),
        ])
        .with(if list.pinned {
            "Unpin from sidebar"
        } else {
            "Pin to sidebar"
        })]),
        a([
            class("block px-4 py-1 border rounded hover:bg-neutral-700 border-neutral-700 w-max"),
            href(format!("/lists/{}/edit_title", list.id)),
        ])
        .with([
            text("Rename"),
            a([
                class(
                    "block px-4 py-1 border rounded hover:bg-neutral-700 border-neutral-700 w-max",
                ),
                href(format!("/bookmarks/create?parent_id={}", list.id)),
                attr("hx-get", format!("/bookmarks/create?parent_id={}", list.id)),
                attr("hx-select", "#create_bookmark"),
                attr("hx-target", "closest section"),
            ])
            .with("Add new bookmark"),
        ]),
    ])
}

fn list_item(link: &LinkWithContent, Data { layout, list, .. }: &Data) -> Element {
    section(class(
        "flex flex-wrap items-end gap-2 px-4 pt-4 pb-4 border-t border-neutral-700",
    ))
    .with([
        div(class("overflow-hidden")).with(match &link.dest {
            db::LinkDestinationWithChildren::List(inner_list) => list_item_list(inner_list),
            db::LinkDestinationWithChildren::Bookmark(bookmark) => list_item_bookmark(bookmark),
        }),
        if let Some(authed_info) = &layout.authed_info {
            div(class(
                "flex flex-wrap justify-end flex-1 pt-2 text-sm basis-32 gap-x-2 text-neutral-400",
            ))
            .with([
                a([
                    class("hover:text-neutral-100"),
                    href(format!("/links/create?dest_id={}", link.dest.id())),
                ])
                .with("Connect"),
                if authed_info.user_id == list.user_id {
                    fragment().with([
                        span(()).with("∙"),
                        button([
                            class("hover:text-neutral-100"),
                            attr("hx-delete", format!("/links/{}", link.id)),
                            attr("title", "Remove from list"),
                        ])
                        .with("Remove from list"),
                    ])
                } else {
                    nothing()
                },
            ])
        } else {
            nothing()
        },
    ])
}

fn list_item_bookmark(bookmark: &db::Bookmark) -> Element {
    fragment().with([
        a([
            class(
                "block overflow-hidden leading-8 text-orange-100 hover:text-orange-300 \
                 text-ellipsis whitespace-nowrap",
            ),
            href(&bookmark.url),
        ])
        .with(&bookmark.title),
        content::link_url(&bookmark.url),
    ])
}

fn list_item_list(inner_list: &db::ListWithLinks) -> Element {
    fragment().with([
        a([
            class(
                "block overflow-hidden font-semibold leading-8 hover:text-fuchsia-300 \
                 text-ellipsis whitespace-nowrap",
            ),
            href(format!("/lists/{}", inner_list.list.id)),
        ])
        .with(&inner_list.list.title),
        fragment().with(inner_list.list.content.as_ref().and_then(|content| {
            (!content.is_empty()).then_some(p(class("max-w-2xl mt-2")).with(content))
        })),
        fragment().with(
            (!inner_list.links.is_empty()).then_some(
                ul(class("flex flex-col mt-2 gap-y-2")).with(
                    inner_list
                        .links
                        .iter()
                        .map(|link| match link {
                            db::LinkDestination::Bookmark(bookmark) => li(()).with([
                                a([
                                    class("block leading-8 text-orange-100 hover:text-orange-300"),
                                    href(&bookmark.url),
                                ])
                                .with(&bookmark.title),
                                content::link_url(&bookmark.url),
                            ]),
                            db::LinkDestination::List(list) => li(()).with([a([
                                class("block leading-8 text-fuchsia-100 hover:text-fuchsia-300"),
                                href(format!("/lists/{}", list.id)),
                            ])
                            .with(format!("{} →", list.title))]),
                        })
                        .collect::<Vec<Element>>(),
                ),
            ),
        ),
    ])
}
