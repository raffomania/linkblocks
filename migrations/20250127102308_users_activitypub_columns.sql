create table ap_users (
    id uuid
        primary key
        default gen_random_uuid()
        not null,
    ap_id varchar(255)
        unique
        not null,
    username varchar(50)
        unique
        not null,
    inbox_url varchar(255)
        not null,
    public_key varchar(10000)
        not null,
    private_key varchar(10000)
        default null,
    last_refreshed_at timestamp with time zone
        not null,
    display_name varchar(100)
        default null,
    bio varchar(1000)
        default null
);

alter table users
    add column ap_user_id uuid
        references ap_users(id)
        default null
;
