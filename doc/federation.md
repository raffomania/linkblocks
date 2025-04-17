# Federation

This document serves as a technical plan for implementing federation in linkblocks, including a survey of how other platforms federate.

## Compatibility

We currently aim for compatibility with Mastodon, Lemmy and Betula. As each of these services has a different feature set, compatibility means a lowest common denominator which both linkblocks and the other service support.

## Users

## Lists

We'll probably build something similar to [Lemmy's groups](https://codeberg.org/fediverse/fep/src/branch/main/fep/1b12/fep-1b12.md).

## Bookmarks

Betula federates bookmarks [as notes](https://git.sr.ht/~bouncepaw/betula/tree/master/item/fediverse/activities/note.go). The bookmark URL is inserted as an `a` tag into the notes' html body, and as a `Link` object into the `attachments` array.

Lemmy's posts are `Page` objects. It seems like mastodon can ingest both `Note` and `Page` objects as toots.

## Comments

## Knowledge Graph

### Lemmy

Links from a group to its entries can be listed through its `replies` field. Links from an entry to its group are established via an entries' `audience` field. Links from an entry to its parent comment are establisthed via its `inReplyTo` field. This forms a tree structure, which may be adapted to a graph by:

- putting lists into the `inReplyTo` field
- putting a link to a collection into the `inReplyTo` field

After a scan of lemmy's code, it seems like it doesn't currently work with either of these approaches: `inReplyTo` will reject anything that is not a single post or a single comment.

Lemmy's cross-posts are a little similar to linkblocks' links between bookmarks and multiple lists, but they are duplicated in lemmy: each group gets its own post, and they are fetched query time in each read operation.

### Ibis Wiki

Links between wiki pages are stored as custom markdown syntax, e.g. ` 	[[Main_Page@ibis.wiki]]`. It seems like they are not represented via any ActivityStreams objects.
