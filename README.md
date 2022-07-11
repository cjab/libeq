[![Crates.io](https://img.shields.io/crates/v/libeq.svg)](https://crates.io/crates/libeq)
[![Docs.rs](https://docs.rs/libeq/badge.svg)](https://docs.rs/libeq)
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# libeq

Libraries and tools for working with EverQuest game data

## Crates
* [libeq_wld](crates/libeq_wld) - Load `.wld` files.
* [libeq_archive](crates/libeq_archive) - Create and extract `.s3d` archives.

## Examples

```rust
use libeq::archive::EqArchive;
use libeq::wld;

fn main() {
    // Extract .wld data from an .s3d file
    let archive = EqArchive::read("gfaydark.s3d").unwrap();
    let (_, data) = archive
        .iter()
        .find(|(name, _)| name == "gfaydark.wld")
        .unwrap();

    // Load .wld file
    let wld = wld::load(data).unwrap();
    let materials = wld.materials().collect::<Vec<_>>();
    let meshes = wld.meshes().collect::<Vec<_>>();
    let models = wld.models().collect::<Vec<_>>();
    let objects = wld.objects().collect::<Vec<_>>();
}
```

## Tools
This workspace also includes the [wld-cli](tools/wld-cli) tool for viewing
fragments within a file. Given a .wld file you're interested in you can view
the fragments with:

```rust
cargo run -p wld-cli -- explore gfaydark.wld
```

Or to extract to raw fragment data files:
```rust
cargo run -p wld-cli -- extract gfaydark.wld destination/
```
