alter table users
    add column email text
        default null,
    add column oidc_id text
        default null,
    alter column username
        drop not null,
    alter column password_hash
        drop not null;
