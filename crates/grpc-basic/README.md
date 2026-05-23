# grpc-basic
Minimal gRPC server/client using tonic + prost.

## Prereqs
- Rust toolchain (stable)
- `protoc` (install on macOS: `brew install protobuf`)

## Run
In one terminal:
```bash
cargo run --bin server
```

In another terminal:
```bash
cargo run --bin client
```

You should see `ok: true` from the client.

## Notes
- Protos live in `proto/hello.proto`.
- Codegen runs via `build.rs`.
