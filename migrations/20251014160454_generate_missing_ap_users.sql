alter table users
    alter column ap_user_id
    set not null;

alter table users
    alter column ap_user_id
    drop default;
