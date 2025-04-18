alter table bookmarks
    add column ap_id
        varchar(255)
        unique
        default null;

-- Because `ap_id` and `id` have to correlate, make sure we
-- always pass in the id explicitly
alter table bookmarks
    alter column id
    drop default;
