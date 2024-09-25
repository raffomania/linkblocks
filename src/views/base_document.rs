use htmf::declare::*;

pub fn base_document<'a>() -> Builder<'a> {
    document()
        .html()
        .class("w-full h-full")
        .with(
            head().with([
                link().rel("stylesheet").href("/assets/preflight.css"),
                link().rel("stylesheet").href("/assets/railwind.css"),
                script().src("/assets/htmx.1.9.9.js"),
                meta().name("color-scheme").content("dark"),
                meta()
                    .name("viewport")
                    .content("width=device-width,initial-scale=1"),
            ]),
        )
        .body()
        .class("w-full h-full text-gray-200 bg-neutral-800")
}
