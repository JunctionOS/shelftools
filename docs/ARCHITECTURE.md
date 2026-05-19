# Architecture

`jiftools` is intentionally small:

```text
jiftools
├── jif            core JIF library
├── tracer-format  Junction trace parser
└── jiftool        single command-line interface
```

The important boundary is library versus CLI. JIF parsing, writing,
transformation, address resolution, and stats live in `jif`. Trace parsing and
trace deduplication live in `tracer-format`. `jiftool` should mostly parse
arguments, call library functions, and format output.

## Core Library

The `jif` crate keeps the format's useful split between raw and materialized
data:

- `JifRaw` is close to the on-disk layout.
- `Jif` resolves references into the shape most commands should use.

Important files:

| Path | Contents |
| --- | --- |
| `jif/src/lib.rs` | Public exports. |
| `jif/src/jif.rs` | `Jif`, `JifRaw`, file-level transforms, summaries, and address lookup helpers. |
| `jif/src/pheader.rs` | VMA/program-header modeling. |
| `jif/src/ord.rs` | Ordering chunks and ordering-section stats. |
| `jif/src/itree/` | Interval trees and interval diffs. |
| `jif/src/read/` | Binary parsers. |
| `jif/src/write/` | Binary serializers. |
| `jif/src/error/` | Error types grouped by format component. |

## CLI

`jiftool` is the only active binary crate. Its structure is:

| Path | Contents |
| --- | --- |
| `jiftool/src/main.rs` | Top-level command routing. |
| `jiftool/src/commands/` | One module per user-facing subcommand. |
| `jiftool/src/io.rs` | Shared file and trace loading/writing helpers. |
| `jiftool/src/digest.rs` | Snapshot comparison and page identity logic. |
| `jiftool/src/python.rs` | Shared Python plotter runner. |
| `jiftool/src/tsa.rs` | Conversion from trace entries to ordering chunks. |

## Data Flow

Reading:

```text
JIF file -> jif/src/read/* -> JifRaw -> Jif -> jiftool output
```

Writing:

```text
jiftool args -> Jif transform -> JifRaw -> jif/src/write/* -> JIF file
```

Trace commands:

```text
trace file -> tracer-format -> Jif address context -> jiftool output
```

## Rules Of Thumb

- If it describes the JIF format or memory contents, put it in `jif`.
- If it describes trace syntax, put it in `tracer-format`.
- If it describes command-line UX, table output, or plotting, put it in
  `jiftool`.
- Prefer explicit CLI subcommands over custom mini-languages.
