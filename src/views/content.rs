use htmf::prelude::*;

pub fn link_url(url: &str) -> Element {
  p(class(
    "w-full max-w-sm overflow-hidden text-sm text-neutral-400 whitespace-nowrap text-ellipsis",
  ))
  .with(url)
}
