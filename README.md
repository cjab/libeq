[![Crates.io](https://img.shields.io/crates/v/libeq.svg)](https://crates.io/crates/libeq)
[![Docs.rs](https://docs.rs/libeq/badge.svg)](https://docs.rs/libeq)
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# libeq

Libraries and tools for working with EverQuest game data

## Crates
* [libeq_wld](crates/libeq_wld) - Load `.wld` files.
* [libeq_pfs](crates/libeq_pfs) - Create and extract `.s3d` archives.

## Examples

```rust
use libeq::pfs::PfsReader;
use libeq::wld;

fn main() {
    // Extract .wld data from an .s3d file
    let file = std::fs::File::open("gfaydark.s3d").unwrap();
    let mut archive = PfsReader::open(file).unwrap();
    let data = archive.get("gfaydark.wld").unwrap().unwrap();

    // Load .wld file
    let wld = wld::load(&data).unwrap();
    let materials = wld.materials().collect::<Vec<_>>();
    let meshes = wld.meshes().collect::<Vec<_>>();
    let models = wld.models().collect::<Vec<_>>();
    let objects = wld.objects().collect::<Vec<_>>();
}
```

## Tools

### s3d
[s3d](tools/s3d) is a CLI tool for listing, extracting, creating, and
verifying EverQuest PFS archives.

```bash
# List files in an archive
s3d list gfaydark.s3d

# Extract all files
s3d extract gfaydark.s3d -o gfaydark/

# Create an archive from a directory
s3d create gfaydark-new.s3d gfaydark/
```

### wld-cli
This workspace also includes the [wld-cli](tools/wld-cli) tool for viewing
and extracting fragments from .wld files.

```bash
# To view fragments
cargo run -p wld-cli -- explore gfaydark.wld

# Extract to raw fragment data files:
cargo run -p wld-cli -- extract gfaydark.wld destination/

# Extract and create to/from RON:
cargo run -p wld-cli -- extract --format ron gfaydark.wld gfaydark.ron
cargo run -p wld-cli -- create --format ron gfaydark.ron gfaydark.wld

# Extract and create to/from JSON:
cargo run -p wld-cli -- extract --format json gfaydark.wld gfaydark.json
cargo run -p wld-cli -- create --format json gfaydark.json gfaydark.wld
```
