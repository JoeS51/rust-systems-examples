# pg_extension

Minimal `pgrx` PostgreSQL extension example in Rust.

## Prerequisites

- Rust toolchain (`rustup`, `cargo`)
- `cargo-pgrx` installed:

```bash
cargo install cargo-pgrx --locked
```

## One-time setup

Initialize `pgrx` for PostgreSQL 15:

```bash
cargo pgrx init --pg15 download
```

This creates the local `pgrx` config at `~/.pgrx/config.toml` and installs a managed PG15 instance.

## Run locally

From this directory (`examples/pg_extension`):

```bash
cargo pgrx run pg15
```

This builds the extension, installs it into the managed PG15 instance, starts PostgreSQL, and opens `psql`.

## In psql

Install and call the extension function:

```sql
CREATE EXTENSION pg_extension;
SELECT hello_pg_extension();
SELECT to_title('my test extension');
```

## Run tests

Use `pgrx` tests with PG15:

```bash
cargo pgrx test pg15
```

If you run `cargo test` directly, use PG15 features explicitly:

```bash
cargo test --no-default-features --features pg15
```
