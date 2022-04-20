[![Crates.io](https://img.shields.io/crates/v/eq_wld.svg)](https://crates.io/crates/eq_wld)
[![Docs.rs](https://docs.rs/eq_wld/badge.svg)](https://docs.rs/eq_wld)

# eq_wld

## An Everquest .wld file loader
This is a work in progress but already loads enough data from wld files to be able to do some
basic rendering of models. The interface has been heavily influenced by the
[glTF](https://github.com/gltf-rs/gltf) crate. Parts of the wld file format are still not well
understood and future understanding may influence the api of this crate.

## Examples
```rust
let archive = eq_archive::read("gfaydark.s3d").unwrap();
let wld_data = archive.get("gfaydark.wld").unwrap();

let wld = eq_wld::load(&wld_data).unwrap();

// Iterate over meshes
for mesh in wld.meshes() {
    let name = mesh.name();
    let positions = mesh.positions();
    let normals = mesh.normals();
    let texture_coordinates = mesh.texture_coordinates();
    let indices = mesh.indices();
    let center = mesh.center();
    ...
}

// Iterate over materials
for material in wld.materials() {
    let name = material.name();
    let texture = material.base_color_texture();
    let texture_source = texture.source();
    ...
}
```

## Tools
This workspace also includes a tool for viewing fragments within a file. It's even more of a work in progress.
Given a world file you're interested in you can view the fragments with:
```
cargo run -p wld-cli -- gfaydark.wld
```

## Acknowledgements
This project wouldn't have been possible without Windcatcher's [WLD File Reference](https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md).

Some documentation has been reproduced as comments within the parser module. Names of file
fragments have been changed when another term from the [glTF reference](https://www.khronos.org/files/gltf20-reference-guide.pdf)
seemed like a better fit. The goal is that this will be usable in more modern engines and
hopefully the names used are more familiar in that context.

