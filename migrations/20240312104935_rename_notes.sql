-- Add migration script here
alter table notes rename to lists;

alter table links rename column src_note_id to src_list_id;
alter table links rename column dest_note_id to dest_list_id;
