# 2.0.2

This version does not contain any functional changes. It only updates third-party dependencies.

# 2.0.1

This is a patch release that does nothing except fixing configs and makes sure the container images are pushed to Docker Hub...

# 2.0.0

This version contains a large refactor to match an internal project template. This results in a few breaking changes and improvements:

- The `--listen_addr`/`LISTEN_ADDR` setting has been renamed to `--listen`/`LISTEN`.
- Log levels and formats are now configurable, see `--help`.
- Binary releases and the official container images switched from Debian to Alpine as the base system. `libjemalloc` is no longer available in the container.

# 1.3.2

This version does not contain any functional changes. It only updates third-party dependencies.

# 1.3.1

This version does not contain any functional changes. It only updates third-party dependencies.

# 1.3.0

This version instroduces a new setting, `--use-x-real-ip`/`USE_X_REAL_IP`, that defaults to `false`. If set, `vssv` will read the `x-real-ip` header to determine the client IP address, which gets stored in the audit log. This is useful, for example, if you run `vssv` behind a reverse proxy like `nginx`.

# 1.2.0

This version introduces a new setting, `--threads`/`THREADS` that allows limiting the number of worker threads and the size of the database connection pool. If this flag is not set, the number of available CPU cores will be used, which matches the current behavior.

# 1.1.3

This version does not contain any functional changes. It only updates third-party dependencies.

# 1.1.2

This version does not contain any functional changes. It only updates third-party dependencies.

# 1.1.1

This version does not contain any functional changes. It only updates third-party dependencies.

# 1.1.0

This release allows using CLI flags in addition to environment variables to configure `vssv`.

# 1.0.0

The first public release. Changes are: everything and nothing.
