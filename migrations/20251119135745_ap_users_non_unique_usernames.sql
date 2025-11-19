alter table ap_users
    drop constraint ap_users_username_key;

create index on ap_users (username);
