# libeq_pfs

PFS (also known as .s3d, .eqg, .pfs) is an archive file format used by the
EverQuest client to store compressed game assets.

This is an implementation of the format as a Rust library crate. There is
a separate [s3d tool](https://github.com/cjab/libeq/tools/s3d) built on top
of it but the PFS format implementation itself can be re-used in other
applications. The API is very similar to other archive crates for Rust
(`tar` and `zip`).

In the future the plan is to also expose the library via a small C wrapper
so that it can be used via FFI in other languages.

## Examples

```rust,no_run
use std::io::Cursor;
use libeq_pfs::PfsReader;
use libeq_pfs::PfsWriter;

let file = std::fs::File::open("gfaydark.s3d").unwrap();

// Open the archive
let mut reader = PfsReader::open(file).unwrap();

// List all files in the archive
let filenames = reader.filenames().unwrap();

// Iterate over files in the archive
let files: Vec<_> = filenames.iter().map(|name| {
    (name, reader.get(name).unwrap(), reader.info(name).unwrap())
}).collect();

// Create a new archive based on an existing one
let new_file = std::fs::File::create("gfaydark-new.s3d").unwrap();
let mut writer = reader.to_writer(new_file).unwrap();

// Add a new file
writer.insert("new-file", Cursor::new(vec![0xde, 0xad, 0xbe, 0xef])).unwrap();
// Finish writing
writer.finish().unwrap();

```

## Why?

Many PFS/S3D extractors and creators have been written over the years.
And these work just fine! The goal of this project however is to build
on that knowledge and create a reference implementation that closely
matches the features likely implemented by the original internal tool.

## Features

* Lazy, Streamed I/O
* Bit-perfect round trips

### Lazy, Streamed I/O

All implementations I've seen require loading the entire contents of the
archive into memory when either reading or writing. On modern machines this
is no problem. But the file format was designed in the days of 64MB of system
memory and memory efficiency was definitely a goal.

This implementation is able to read and write files to/from the archive
one 8KB block at a time. This means that you can load individual files
from the archive leaving the others on disk. Or even stream/extract file
data from one file to another 8KB at a time, never loading the full file
into memory.

### Bit-perfect Round Trips

If you open an archive, run it through the parser and write it out again you
will get the exact same thing that you put in, at the bit level. This is _very_
helpful for testing correctness. Modifying the archive (adding/removing files)
of course will change the archive but this property also makes it much easier to
view a diff between the original and the new.

## Format

I've also made an effort to document the PFS format in [FORMAT.md](./FORMAT.md)
