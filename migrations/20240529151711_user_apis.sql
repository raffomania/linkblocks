create table user_apis (
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
    api_key text
        not null,
    permissions text
        default 'Add',
    valid_until timestamp with time zone
        not null
);