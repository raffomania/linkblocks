use htmf::prelude::*;

pub fn link_url(url: &str) -> Element {
    p(class(
        "w-full max-w-sm overflow-hidden text-sm text-neutral-400 whitespace-nowrap text-ellipsis",
    ))
    .with(url)
}

pub fn pluralize<'a>(
    count: i64,
    singular_description: &'a str,
    plural_description: &'a str,
) -> String {
    match count {
        1 => format!("{count} {singular_description}"),
        _ => format!("{count} {plural_description}"),
    }
}
