# Rust Systems Examples
Rust implementations of different systems concepts.

This repo is a concept lab: each example is intentionally minimal and self-contained so you can learn a topic quickly, run it, and reference it later.

## Structure
- `examples/` contains individual, runnable projects.
- `crates/` is reserved for shared libraries or future experiments.

## Getting started
Prereqs:
- Rust toolchain 
- Additional tools as noted per example (e.g., `protoc` for gRPC)

Run an example:
```bash
cd examples/<example-name>
cargo run --bin <bin-name>
```

If you want to run from anywhere:
```bash
cargo run --manifest-path /path/to/examples/<example-name>/Cargo.toml --bin <bin-name>
```

## Examples
- `examples/grpc-basic`: minimal gRPC server/client using tonic + prost. See `examples/grpc-basic/README.md`.
- `examples/websockets`: planned websocket example. See `examples/websockets/README.md`.

## Notes
- Examples favor clarity and small size over completeness.
- If an example uses codegen, check its `build.rs` or README for tool requirements.
- rust-analyzer makes it easy to inspect generated code via “Go to Definition” on types from `tonic::include_proto!`.
