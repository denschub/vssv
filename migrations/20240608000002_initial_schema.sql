create table secrets (
  uuid uuid primary key default uuid_generate_v4(),

  created_at timestamp with time zone not null default now(),
  updated_at timestamp with time zone not null default now(),

  file_name text,
  contents bytea,

  notes text
);
select manage_updated_at('secrets');

create table tokens (
  uuid uuid primary key default uuid_generate_v4(),

  created_at timestamp with time zone not null default now(),
  updated_at timestamp with time zone not null default now(),
  used_at timestamp with time zone,
  expires_at timestamp with time zone,

  token char(48) not null default encode(gen_random_bytes(24), 'hex'),
  superuser boolean not null default false,

  notes text
);
select manage_updated_at('tokens');

create table token_permissions (
  token uuid not null,
  secret uuid not null,

  created_at timestamp with time zone not null default now(),
  updated_at timestamp with time zone not null default now(),

  can_read boolean not null default false,
  can_write boolean not null default false,

  notes text,

  primary key(token, secret),
  foreign key(token) references tokens(uuid) on delete cascade,
  foreign key(secret) references secrets(uuid)  on delete cascade
);
select manage_updated_at('token_permissions');

create type audit_log_action as enum ('secret_read', 'secret_write');
create table audit_log (
  entry uuid not null default uuid_generate_v4(),

  event_ts timestamp with time zone not null default now(),
  client_addr inet not null,

  action audit_log_action not null,
  token uuid not null,
  secret uuid not null
);
