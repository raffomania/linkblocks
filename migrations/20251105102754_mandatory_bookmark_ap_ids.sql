alter table bookmarks
    alter column ap_id
    set not null;

alter table bookmarks
    alter column ap_id
    drop default;
