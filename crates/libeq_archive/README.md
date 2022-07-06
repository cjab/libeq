# eq_archive

## An Everquest archive file extractor
This has only been tested on .s3d files and implements only the bare minimum of functionality.
CRC checks for example are completely ignored.

## Examples
```rust
let archive = eq_archive::read("gfaydark.s3d").unwrap();

// List all files in the archive
let filenames = archive.filenames();

// Iterate over files in the archive
for (name, data) in archive.files() {

}

```

