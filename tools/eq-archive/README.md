eq-archive
==========

A tool for working with EverQuest .s3d archives.

## Use
```
# To extract files from an .s3d archive
cargo run -- -x fixtures/gfaydark.s3d gfaydark

# To create a .s3d archive from a directory
cargo run -- -c gfaydark gfaydark.s3d
```
