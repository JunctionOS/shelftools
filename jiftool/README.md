# `jiftool`

One CLI for working with JIF files.

## Commands

```sh
jiftool read <JIF> [summary|pheaders|ord|raw]
jiftool check [--raw] <JIF>
jiftool modify <INPUT> <OUTPUT> <operation>
jiftool trace <JIF> <TRACE>
jiftool compare [OPTIONS] <JIF> <JIF>...
jiftool time <JIF> <TRACE> <OUTPUT>
```

## Examples

Inspect a file:

```sh
jiftool read image.jif
jiftool read image.jif pheaders --start 0 --end 5
jiftool read image.jif ord
jiftool read image.jif raw summary
```

Validate parsing:

```sh
jiftool check image.jif
jiftool check --raw image.jif
```

Modify a file:

```sh
jiftool modify input.jif output.jif rewrite
jiftool modify input.jif output.jif rename /usr/bin/ld.so /bin/ld.so
jiftool modify input.jif output.jif build-itrees
jiftool modify input.jif output.jif add-ord trace.ord
```

Work with traces:

```sh
jiftool trace image.jif trace.ord
jiftool time image.jif trace.ord access-plot
```

Compare snapshots:

```sh
jiftool compare a.jif b.jif
jiftool compare --private a.jif b.jif
jiftool compare --ordering --output upset.pdf a.jif b.jif c.jif
```
