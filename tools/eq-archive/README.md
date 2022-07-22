eq-archive
==========

A tool for working with EverQuest .s3d and .pfs archives.

## Use
```bash
# To extract files from an archive
cargo run -- -x fixtures/gfaydark.s3d gfaydark
cargo run -- -x fixtures/snd1.pfs snd1

# To create an archive from a directory
cargo run -- -c gfaydark gfaydark.s3d
cargo run -- -c snd1 snd1.pfs
```
