//! Utilities for converting between [`usize`]s and fixed-size integrals in wgpu, mostly
//! [`usize_from_u32`].
//!
//! wgpu crates only support 32-bit targets. This module both enforces this at compile time, and
//! offers conveniences for converting between [`u32`] and [`usize`] that [`core`] cannot provide,
//! because Rust's minimum assumed word size is 16 bits.

/// This is the load-bearing constant type assertion for the rest of the module. If this is true,
/// then everything else should Just Work™.
const _: () = assert!(
    size_of::<usize>() >= size_of::<u32>(),
    "word sizes < 32 bits are not supported in wgpu crates"
);

/// Convert a [`u32`] to a [`usize`].
pub const fn usize_from_u32(n: u32) -> usize {
    n as usize
}
