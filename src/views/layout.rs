use htmf::declare::*;

use crate::{
    authentication::AuthUser,
    db::{self, layout::AuthedInfo, AppTx, List},
    response_error::ResponseResult,
};

use super::base_document::base_document;

pub struct Template {
    pub authed_info: Option<AuthedInfo>,
}

impl Template {
    pub async fn from_db(tx: &mut AppTx, auth_user: Option<&AuthUser>) -> ResponseResult<Self> {
        let auth_info = if let Some(auth_user) = auth_user {
            Some(db::layout::by_user_id(tx, auth_user.user_id).await?)
        } else {
            None
        };
        Ok(Template {
            authed_info: auth_info,
        })
    }
}

pub fn layout<'a>(children: Builder<'a>, layout: &'a Template) -> Builder<'a> {
    base_document()
        .div(class("flex-row-reverse h-full sm:flex"))
        .with([
            main_(class("sm:overflow-y-auto sm:grow")).with(children),
            match &layout.authed_info {
                Some(info) => sidebar(info),
                None => fragment(),
            },
        ])
}

fn sidebar(authed_info: &AuthedInfo) -> Builder<'_> {
    aside(id("nav").class(
        "bg-neutral-900 sm:max-w-[18rem] sm:w-1/3 sm:max-h-full flex flex-col sm:flex-col-reverse \
         sm:border-r border-neutral-700 border-t sm:border-t-0",
    ))
    .with([
        div(class("sm:overflow-y-auto sm:flex-1")).with([lists_header(), lists(authed_info)]),
        header(class(
            "sticky bottom-0 flex justify-between p-2 leading-8 bg-neutral-900",
        ))
        .with([
            a(href("/").class("px-2 font-bold rounded  hover:bg-neutral-800"))
                .text(&authed_info.user_description),
            form(action("/logout").method("post")).with(
                button(class("rounded px-3  text-neutral-400 hover:bg-neutral-800")).text("Logout"),
            ),
        ]),
    ])
}

fn lists_header() -> Builder<'static> {
    div(class(
        "sticky top-0 flex items-center justify-between px-2 pt-2 sm:top-0 bg-neutral-900",
    ))
    .with([
        h3(class(
            "px-2 py-1 text-sm font-bold tracking-tight text-neutral-400",
        ))
        .text("Lists"),
        a(href("/lists/create")
            .class("block px-3 text-xl rounded hover:bg-neutral-800 text-neutral-400"))
        .text("+"),
    ])
}

fn lists(authed_info: &AuthedInfo) -> Builder {
    let lists = authed_info.lists.iter();
    ul(class("pb-2")).with([
        li([])
            .a(class(
                "block px-4 py-1 overflow-hidden text-ellipsis whitespace-nowrap \
                 hover:bg-neutral-800",
            )
            .href("/bookmarks/unsorted"))
            .text("Unsorted bookmarks"),
        fragment().with(lists.map(list_item).collect::<Vec<_>>()),
        li([])
            .a(class(
                "block px-4 py-1 overflow-hidden text-ellipsis whitespace-nowrap \
                 hover:bg-neutral-800 text-neutral-400",
            )
            .href("/lists/unpinned"))
            .text("Unpinned lists"),
    ])
}

fn list_item(list: &List) -> Builder {
    li([])
        .a(class(
            "block px-4 py-1 overflow-hidden text-ellipsis whitespace-nowrap hover:bg-neutral-800",
        )
        .href(format!("/lists/{}", list.id)))
        .text(&list.title)
}
