create table follows (
    id uuid primary key
        default gen_random_uuid()
        not null,
    follower_id uuid
        references ap_users(id)
        not null,
    following_id uuid
        references ap_users(id)
        not null,

    unique (follower_id, following_id)
);

-- Update wrong activitypub inbox paths for local users
update ap_users
    set inbox_url = replace(inbox_url, '/ap/inbox', '/ap/inbox/' || ap_users.id)
    where exists (
        select 1
        from users
        where users.ap_user_id = ap_users.id
            and ap_users.inbox_url like '%/ap/inbox'
    )
;

