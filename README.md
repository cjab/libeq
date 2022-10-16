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

### eq-archive
[eq-archive](tools/eq-archive) is a cli tool used to extract and
create EverQuest .s3d archives.

```bash
# To extract files from an archive
cargo run -p eq-archive -- -x fixtures/gfaydark.s3d gfaydark

# To create an archive from a directory
cargo run -p eq-archive -- -c gfaydark gfaydark.s3d
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
