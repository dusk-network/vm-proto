#![cfg_attr(not(feature = "host"), no_std)]
#![cfg_attr(not(feature = "host"), feature(alloc_error_handler))]

mod definitions;
pub use definitions::*;

#[cfg(feature = "host")]
mod host;

#[cfg(feature = "host")]
pub use host::*;

pub mod abi;

#[cfg(not(feature = "host"))]
mod no_std_plumbing;
