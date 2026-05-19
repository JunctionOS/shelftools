# `jiftools`

Rust library and one command-line tool for reading, modifying, comparing, and
tracing JIF files (Junction Image Format).

The active workspace has three crates:

| Crate | Purpose |
| --- | --- |
| [`jif`](jif/README.md) | Core JIF data model, parser, writer, and transformations. |
| [`jiftool`](jiftool/README.md) | Single CLI for inspection, mutation, comparison, trace context, and plotting. |
| [`tracer-format`](tracer-format/README.md) | Parser for Junction timestamped access traces. |

The JIF format specification is reserved for [`SPEC.md`](SPEC.md); that file is
currently a placeholder. For code layout, start with
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Quick Start

Build everything:

```sh
cargo build --workspace
```

Run the test suite:

```sh
cargo test --workspace
```

Inspect a JIF:

```sh
cargo run -p jiftool -- read image.jif
cargo run -p jiftool -- read image.jif pheaders --start 0 --end 5
cargo run -p jiftool -- read image.jif ord
```

Check that a file parses:

```sh
cargo run -p jiftool -- check image.jif
cargo run -p jiftool -- check --raw image.jif
```

Write a modified JIF:

```sh
cargo run -p jiftool -- modify input.jif output.jif build-itrees
cargo run -p jiftool -- modify input.jif output.jif rename /usr/bin/ld.so /bin/ld.so
cargo run -p jiftool -- modify input.jif ordered.jif add-ord trace.ord
```

Trace and compare:

```sh
cargo run -p jiftool -- trace image.jif trace.ord
cargo run -p jiftool -- compare a.jif b.jif
cargo run -p jiftool -- time image.jif trace.ord access-plot
```

## Where Things Live

- `jif/src/jif.rs` owns `Jif`, `JifRaw`, whole-file operations, summaries, and
  address resolution helpers.
- `jif/src/ord.rs` owns ordering chunks and ordering-section stats.
- `jif/src/itree/` owns interval-tree data structures and transformations.
- `jif/src/read/` and `jif/src/write/` own binary parsing and serialization.
- `jiftool/src/main.rs` owns top-level command routing.
- `jiftool/src/commands/` owns user-facing behavior for each subcommand.
- `jiftool/src/io.rs`, `digest.rs`, and `python.rs` contain reusable CLI
  helpers.
- `tracer-format/src/` owns trace parsing and deduplication.
