create table bookmarks (
    id uuid
        primary key
        default gen_random_uuid()
        not null,
    created_at timestamp with time zone
        default current_timestamp
        not null,
    user_id uuid
        references users(id)
        not null,

    url text
        not null,
    title text
        not null
);

create table notes (
    id uuid
        primary key
        default gen_random_uuid()
        not null,
    created_at timestamp with time zone
        default current_timestamp
        not null,
    user_id uuid
        references users(id)
        not null,

    content text
        not null
);

create table lists (
    id uuid primary key
        default gen_random_uuid()
        not null,
    created_at timestamp with time zone
        default current_timestamp
        not null,
    user_id uuid
        references users(id)
        not null,

    title text
        not null
);

create table links (
    id uuid
        primary key
        default gen_random_uuid()
        not null,
    created_at timestamp with time zone
        default current_timestamp
        not null,
    user_id uuid
        references users(id)
        not null,

    src_bookmark_id uuid
        references bookmarks(id)
        default null,
    src_note_id uuid
        references notes(id)
        default null,
    src_list_id uuid
        references lists(id)
        default null,

    check (
        num_nonnulls(
            src_bookmark_id, src_note_id, src_list_id
        ) = 1
    ),

    dest_bookmark_id uuid
        references bookmarks(id)
        default null,
    dest_note_id uuid
        references notes(id)
        default null,
    dest_list_id uuid
        references lists(id)
        default null,

    check (
        num_nonnulls(
            dest_bookmark_id, dest_note_id, dest_list_id
        ) = 1
    )
)
