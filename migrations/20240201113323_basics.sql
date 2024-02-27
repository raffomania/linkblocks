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

    title text
        not null,
    content text
        default null
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

    src_note_id uuid
        references notes(id)
        not null,

    dest_bookmark_id uuid
        references bookmarks(id)
        default null,
    dest_note_id uuid
        references notes(id)
        default null,

    check (
        num_nonnulls(
            dest_bookmark_id, dest_note_id
        ) = 1
    )
)
