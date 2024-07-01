alter table users
    add column email text
        default null,
    add column using_oauth boolean
        not null
        default false,
    add column oauth_provider text
        default null,
    add column oauth_id text
        default null;
