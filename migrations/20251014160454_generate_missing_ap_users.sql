alter table users
    alter column ap_user_id
    set not null;

alter table users
    alter column ap_user_id
    drop default;

-- Because `ap_id` and `id` have to correlate for local users, make sure we
-- always pass in the id explicitly
alter table ap_users
    alter column id
    drop default;
