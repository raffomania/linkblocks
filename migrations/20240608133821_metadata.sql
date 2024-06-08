create table metadata(
    id uuid primary key
        default gen_random_uuid()
        not null,
    metadata_title text
        not null,
    metadata_description text,
    metadata_image_url text
);

alter table bookmarks
    add column metadata_id uuid
        references metadata(id);

alter table lists
    add column rich_view boolean
        default false
        not null;