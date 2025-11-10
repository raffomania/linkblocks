-- add optional column
alter table bookmarks
    add column ap_user_id uuid
        references ap_users(id)
        default null
;

-- populate column
update bookmarks
    set ap_user_id = ap_users.id
    from ap_users
        join users on users.ap_user_id = ap_users.id
    where users.id = bookmarks.user_id
;

-- make it non-optional
alter table bookmarks
    alter column ap_user_id
    set not null
;

alter table bookmarks
    alter column ap_user_id
    drop default
;

-- drop old column
alter table bookmarks
    drop column user_id
;

-- lists

-- add optional column
alter table lists
    add column ap_user_id uuid
        references ap_users(id)
        default null
;

-- populate column
update lists
    set ap_user_id = ap_users.id
    from ap_users
        join users on users.ap_user_id = ap_users.id
    where users.id = lists.user_id
;

-- make it non-optional
alter table lists
    alter column ap_user_id
    set not null
;

alter table lists
    alter column ap_user_id
    drop default
;

-- drop old column
alter table lists
    drop column user_id
;
