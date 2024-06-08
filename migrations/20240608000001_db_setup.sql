create extension if not exists "uuid-ossp";
create extension if not exists "pgcrypto";

create or replace function manage_updated_at(_tbl regclass) returns void as $$
begin
  execute format(
    'create trigger set_updated_at before update on %s
    for each row execute procedure set_updated_at()',
    _tbl
  );
end;
$$ language plpgsql;

create or replace function set_updated_at() returns trigger as $$
begin
  if (new is distinct from old and new.updated_at is not distinct from old.updated_at) then
    new.updated_at := current_timestamp;
  end if;
  return new;
end;
$$ language plpgsql;
