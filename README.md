# vssv

(That stands for **v**ery **s**imple **s**ecret **v**ault. Creative name, I know.)

A simple service that stores secrets (in the form of files that can be downloaded) and delivers them over an HTTP API. It's better than checking in your secrets into version control, but that's about it. Secrets are stored in a PostgreSQL database, and authentication is done via auth tokens, where each token is explicitly granted access to individual secrets.

## You don't want to use this.

No, seriously. You don't want to use this.

1. There is no admin interface. Creating secrets, tokens, and granting permissions needs to happen with direct database access - either via the `psql` CLI, or a database management UI.
2. There is no encryption at rest. All secrets are stored as plain-bytes inside the PostgreSQL database. Everyone with access to the DB has access to everything.
3. There is no audit log for management actions. While there is an audit log for actions via the HTTP API, there are no logs for creating tokens or granting permissions.
4. There is no IP-based allowlist for tokens. Use your server's firewall!
5. No security audit has ever been performed. This application might leak all your secrets if a kitten purrs at it, and you won't even know! Also, this project has ZERO test coverage! Super amateurish!

## HTTP API usage

The HTTP API allows clients to read secrets, and update their contents (but not a secret's metadata).

### Receiving a secret

Example curl call:

```sh
curl -JOH "Authorization: Bearer TOKEN" https://wow-so-secure.exmaple.com/secret/UUID
```

For reference, the combination of curl's `-J` and `-O` tells curl to download the file, and use the file name provided by the server. Providing a file name in the database is optional, but if you do, it will set the right HTTP header to amke that happen.

### Updating a secret's contents

There is no API to update any of a secret's metadata, but you can update a secret's contents. THis is done via a simple HTTP POST:

```sh
curl -X POST --data-binary "@example.json" -H "Authorization: Bearer TOKEN" http://localhost:3000/secret/UUID/contents
```

## Management

There is no UI or CLI. Use a PostgreSQL shell or a database UI to manage `vssv`.

### Managing secrets

All fields in the `secrets` table are either optional, or autogenerated. To create a new secret, you can insert nothing into the table:

```
vssv=# insert into secrets default values returning *;
-[ RECORD 1 ]------------------------------------
uuid       | c86deaa4-513a-4aef-b62d-4bfe8c9ea80b
created_at | 2024-06-08 19:28:56.691145+00
updated_at | 2024-06-08 19:28:56.691145+00
file_name  |
contents   |
notes      |

INSERT 0 1
```

You can then use that UUID and a valid token to push something into it. I strongly recommend setting `file_name` to an actual file name, as that makes downloading easier. The `notes` field is for whatever you want to put into it, it has no actual use. The `updated_at` column is updated automatically every time you change the row.

The `contents` field is of type `bytea`. [Consult the PG documentation](https://www.postgresql.org/docs/current/datatype-binary.html) for how to properly query and store that.

### Managing tokens

Similar to `secrets`, there is an attempt to have reasonable default values, so you can just create an empty row:

```
vssv=# insert into tokens default values returning *;
-[ RECORD 1 ]------------------------------------------------
uuid       | 26ebbc04-ed79-491a-89e8-61413fdbf491
created_at | 2024-06-08 19:31:23.429766+00
updated_at | 2024-06-08 19:31:23.429766+00
used_at    |
expires_at |
token      | 1723a52cfb0deb7ee9225c4ec21d9a811d725a600b3534b8
superuser  | f
notes      |

INSERT 0 1
```

This creates a valid token without any permissions and without an expiratoin. The `token` value itself is the hex representation of 24 bytes of cryptographically strong random data [generated by `pgcrypto`s `gen_random_bytes()` function](https://www.postgresql.org/docs/current/pgcrypto.html#PGCRYPTO-RANDOM-DATA-FUNCS). It should be safe to use. If you generate your own tokens, make sure they're always 48 characters long.

If `expires_at` is set, the token will be rejected after that time. However, the `used_at` timestamp will still be updated, even if an expired token is used. This means that you can use `select uuid from tokens where expires_at is not null and used_at > expires_at` to find cases where you really should fix your deployments.

Just as with `secrets`, the `updated_at` timestamp is updated automatically for your convienience every time you change the row.

If you set `superuser` to `true`, the token will have full read and write permissions to all secrets in the database. Handle with care.

### Granting permissions

Unless a token is a `superuser`, it can neither read nor write anything. That's surprisingly useless, so make sure to grant the tokens you want to use permissions.

If you, for example, want the token we created in this documentation read-only access to the secret we created earlier, use

```
vssv=# insert into token_permissions (token, secret, can_read) values ('26ebbc04-ed79-491a-89e8-61413fdbf491', 'c86deaa4-513a-4aef-b62d-4bfe8c9ea80b', true) returning *;
-[ RECORD 1 ]------------------------------------
token      | 26ebbc04-ed79-491a-89e8-61413fdbf491
secret     | c86deaa4-513a-4aef-b62d-4bfe8c9ea80b
created_at | 2024-06-08 19:40:17.264361+00
updated_at | 2024-06-08 19:40:17.264361+00
can_read   | t
can_write  | f
notes      |

INSERT 0 1
```

## Deployment and configuration

First, scroll back up and re-read the "You don't want to use this." section.

But if you have to, there is a container image [in the GitHub Container registry at `ghcr.io/denschub/vssv:latest`](https://github.com/denschub/vssv/pkgs/container/vssv), and [on Docker Hub as `denschub/vssv:latest`](https://hub.docker.com/repository/docker/denschub/vssv/general). The container exposes port 3000.

Make sure to set the `DATABASE_URL` environment variable to a valid PostgreSQL connection URL like `postgres://postgres@127.0.0.1/vssv`. The database needs to exist before starting the server, but the server startup procedure will take care of all database migrations.

Released binaries are available for all stable releases. Check the [Releases section on GitHub](https://github.com/denschub/vssv/releases) for the latest release, and you'll find a `.zip` with a pre-built binary. If you run the binary yourself, also make sure to set the `[::1]:3000` environment variable to a valid listen address, like `[::1]:3000`, for example.

You can also build a binary yourself if you have the latest stable Rust toolchain installed. Simply run `cargo build --release`, and you'll find a ready-to-use binary at `target/release/vssv`.

## License

[MIT](/LICENSE). But you shouldn't care because you shouldn't use this.
