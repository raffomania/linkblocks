create table users (
    id uuid primary key default gen_random_uuid() not null,
    password_hash text not null,
    username text not null
);
