# libeq_archive

PFS (also known as .s3d) is an archive file format used by the EverQuest
client to store compressed game assets.

This is an implementation of the format as a Rust library crate. There is
a separate [s3d tool](https://github.com/cjab/libeq/tools/s3d) built on top
of it but the PFS format implementation itself can be re-used in other
applications. The API is very similar to other archive crates for Rust
(`tar` and `zip`).

In the future the plan is to also expose the library via a small C wrapper
so that it can be used via FFI in other languages.

## Examples

```rust
use libeq_archive::EqArchiveReader;
use libeq_archive::EqArchiveWriter;

let file = std::fs::File::open("gfaydark.s3d").unwrap();

// Open the archive
let mut reader = EqArchiveReader::open(file).unwrap();

// List all files in the archive
let filenames = reader.filenames().unwrap();

// Iterate over files in the archive
let files: Vec<_> = filenames.iter().map(|name| {
    (name, reader.get(name).unwrap(), reader.info(name).unwrap())
}).collect();

// Modify the archive (gfaydark.s3d)
let mut writer = reader.to_writer().unwrap();

// Add a new file
writer.push("new-file", [0xde, 0xad, 0xbe, 0xef]);
// Remove a file
writer.remove("palette.bmp");
// Finish writing
writer.finish();

```

## Why?

Many PFS/S3D extractors and creators have been written over the years.
And most of these work just fine! The goal of this project however is to
build on the knowledge from these implementations and build a reference
implementation that closely matches the features likely implemented by the
original internal tool.

## Features

* Lazy, Streamed I/O
* Bit-perfect round trips

### Lazy, Streamed I/O

All implementations I've seen required loading the entire contents of the
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
view a diff between the original archive and the new archive.
