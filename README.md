# linkblocks

**📚 A federated network to bookmark, share and discuss good web pages with your friends.**

It's getting harder and harder to find good web pages. When you do find good ones, it's worth hanging onto them. Linkblocks is your own small corner of the web, where you can keep your favorite pages, and share them with your friends to help them find good web pages too.

🔭 Linkblocks is in an exploratory phase where we're trying out different ways to make it work well. You can try it out, but big and small things might change with every update.

## Vision

- On linkblocks, you can organize, connect, browse and search your favorite web pages.
- Share carefully curated or wildly chaotic collections of the stuff you really really like with other linkblocks users and the whole world wide web.
- Follow users with a similar taste and get a feed of fresh good web pages every day. Browse others' collections to discover new web pages from topics you like.
- Annotate, highlight and discuss web pages together with your friends.
- Mark users as trusted whose standards for web pages match yours - and then search through all trusted bookmarks to find good pages on a specific topic. Add trusted users of your trusted users to your search range to cast a wider net.

[See this blog post for more on the vision behind linkblocks.](https://www.rafa.ee/articles/introducing-linkblocks-federated-bookmark-manager/)

## Related Reading

- [Where have all the Websites gone?](https://www.fromjason.xyz/p/notebook/where-have-all-the-websites-gone/) talks about the importance of website curation. Linkblocks is for publicly curating websites.
- [The Small Website Discoverability Crisis](https://www.marginalia.nu/log/19-website-discoverability-crisis/) similar to the previous link, it encourages everyone to share reading lists. By the author of the amazing [marginalia search engine](https://search.marginalia.nu/).

## Development Setup

Install the dependencies:

- [Latest stable version of Rust](https://www.rust-lang.org/learn/get-started) (An older version might work as well, but is not tested)
- [mkcert](https://github.com/FiloSottile/mkcert#installation)
  - Don't forget to run `mkcert -install`
- Optional: [podman](http://podman.io/docs/installation), for conveniently running postgres for development and tests
- Optional: [npm](https://nodejs.org/en/download/package-manager) or a compatible package manager, to format template files

Install dependencies available via cargo:

```sh
cargo install cargo-run-bin
```

Copy `.env.example` to `.env` and edit it to your liking.

Optional: run `just install-git-hooks` to automatically run checks before committing.

In the root of the repository, launch the server:

```sh
cargo bin just watch
```

Then, open [http://localhost:4040] in your browser.

## Hosting Your Own Instance

⚠️ linkblocks is in a pre-alpha stage. There are no versions and no changelog. Expect data loss.

You can run the container at `ghcr.io/raffomania/linkblocks:latest`. It's automatically updated to contain the latest version of the `main` branch.

Linkblocks is configured through environment variables or command line options.
Run `linkblocks --help` to for documentation on the available options.
The [.env.example] file contains an example configuration for a development environment.

## Technical Details

This web app is implemented using technologies hand-picked for a smooth development and deployment workflow. Here are some of the features of the stack:

- Type-safe and fast, implemented in [Rust](https://www.rust-lang.org/) using the [axum framework](https://github.com/tokio-rs/axum)
- Snappy interactivity using [htmx](https://htmx.org/) with almost zero client-side code
- [Tailwind styles without NodeJS](https://github.com/pintariching/railwind), integrated into the cargo build process using [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- Compile-time verified HTML templates using [Askama](https://github.com/djc/askama)
- Compile-time verified database queries using [SQLx](https://github.com/launchbadge/sqlx)
- Concurrent, isolated integration tests with per-test in-memory postgres databases
- Single-binary deployment; all assets baked in
- Integrated TLS; can run without a reverse proxy
- PostgreSQL as the only service dependency
- Built-in CLI for production maintenance
- Auto-reload in development [without dropped connections](https://github.com/mitsuhiko/listenfd)
