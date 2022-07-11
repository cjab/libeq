//! Libraries and tools for working with EverQuest game data
//!
//! # Examples
//!
//! ```rust
//! use libeq::archive::EqArchive;
//! use libeq::wld;
//!
//! fn main() {
//!     // Extract .wld data from an .s3d file
//!     let archive = EqArchive::read("gfaydark.s3d").unwrap();
//!     let (_, data) = archive
//!         .iter()
//!         .find(|(name, _)| name == "gfaydark.wld")
//!         .unwrap();
//!
//!     // Load .wld file
//!     let wld = wld::load(data).unwrap();
//!     let materials = wld.materials().collect::<Vec<_>>();
//!     let meshes = wld.meshes().collect::<Vec<_>>();
//!     let models = wld.models().collect::<Vec<_>>();
//!     let objects = wld.objects().collect::<Vec<_>>();
//! }
//! ```
//!
//! # Crates
//! * [libeq_wld](crates/libeq_wld) - Load `.wld` files.
//! * [libeq_archive](crates/libeq_archive) - Create and extract `.s3d` archives.
//!
//! # Tools
//! * [wld-cli](tools/wld-cli) - Command line tools for working with `.wld` files.
//!

#[cfg(feature = "archive")]
pub use libeq_archive as archive;
#[cfg(feature = "wld")]
pub use libeq_wld as wld;
