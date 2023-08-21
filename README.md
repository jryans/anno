# Anno

Anno is a file annotation toolkit that assists with aggregating annotations and
presenting them in a `git annotate`-style sidebar view.

Annotations come from separate "producer" commands. Anno is packaged with a few
such producers, but you can also write your own without needing to change Anno
itself. Producers are just programs named `anno-<producer-name>` that follow
certain input and output conventions.

## Installation

Anno is still in an initial development phase, so the only installation option
involves building from source. If you have Rust's `cargo` package manager
[installed][install-rust], you can use the following command:

```
$ cargo install --git https://github.com/jryans/anno
```

## Usage

To use Anno, you supply a file to be annotated along with one or more annotation
producers you are interested in. For example, the following uses the simple
`lines` producer which adds lines numbers:

```
$ anno src/main.rs -p numbers:
num |
1   | use std::{
2   |     cmp::max,
3   |     fs,
4   |     path::PathBuf,
5   |     str::{FromStr, Lines},
6   | };
7   |
8   | use anyhow::{Context, Error, Ok, Result};
9   | use clap::Parser;
10  | use duct::cmd;
...
```

Multiple producers can be supplied as well. The annotations from each producer
are aggregated, with producer's data shown in a separate column:

```
$ anno src/main.rs -p debug-line-table:<path-to-debug-info> -p numbers:
...
  | 51  |     // Read the file first to check line count
x | 52  |     let target_path = cli.file.absolutize()?;
x | 53  |     let target_content = fs::read_to_string(&target_path).with_context(|| {
x | 54  |         format!(
  | 55  |             "Unable to read file to be annotated ({})",
x | 56  |             cli.file.display()
  | 57  |         )
x | 58  |     })?;
...
```

## Additional features

- Diff mode (`--diff`)\
  Visually highlights differences between annotations from the first two
  producers

  <img
    width="381"
    alt="Example of diff mode with visual highlighting to denote differences in annotations"
    src="https://github.com/jryans/anno/assets/279572/5fabdaac-4861-467a-adb9-b8ac139f10d8">

- Diff only mode (`--diff-only`)\
  Only show lines with differences between the first two producers

## Included producers

### Numbers

Usage: `-p numbers:`

This producer numbers the lines of the annotated file in order, just like you
might see in an editor.

### Debug line table

Usage: `-p debug-line-table:<path-to-debug-info`

This producer takes a DWARF debug info file and checks whether each source line
of the file being annotated is present in the debug info's line table.

### Source computation

Usage: `-p source-computation:`

This producer analyses C source files and annotates source lines where some
notion of "computation" occurs.

You will need to have `dbgcov-cflags` from [`dbgcov`][dbgcov] in your `PATH`.
The `CC` environment variable (or `cc` command) will need to point to a version
of `cc` from GCC.

## Producer URI syntax

Producers are currently enabled via the `-p` option which accepts a URI-based
syntax. There may be better ways to configure this... Feel free to open an issue
if you have a suggestion.

- Producer only\
  `producer:`
- Producer with data source\
  `producer:/path/to/data/source`
- Producer with data source and additional arguments\
  `producer:/path/to/data/source?param=value`

One small oddity of this URI syntax is that you must add a trailing `:` when
naming only the producer. A future version of Anno may make this optional.

## Producer protocol

### Command

For each producer, Anno attempts to execute a command formed from the producer
name with the prefix `anno-` added to it. So for the producer `lines`, Anno
tries to run the command `anno-lines`.

Currently there is no way to supply the full path to a producer command, so
`anno-<producer>` must be accessible via your `PATH` environment variable.

### Input

Producers receive input from Anno via various `ANNO`-prefixed environment
variables. This is still in flux, so it's best to check the source and examples
for now.

### Output

Anno currently uses a very simple annotation format. Each producer supplies
their annotation data as a line of text for each line in the file being
annotated. Other output formats [may be added](#future-work) in the future.

## Future work

- [ ] Add incremental output format
- [ ] Add editor integration with more complex output abilities
- [ ] Add [Compiler Explorer][ce] integration

[install-rust]: https://www.rust-lang.org/tools/install
[dbgcov]: https://github.com/stephenrkell/dbgcov
[ce]: https://github.com/compiler-explorer/compiler-explorer
