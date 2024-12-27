use core::fmt;

use serde::{Deserialize, Serialize};

mod json;
pub use json::Packer as Json;
#[cfg(feature = "msgpack")]
mod msgpack;
#[cfg(feature = "msgpack")]
pub use msgpack::Packer as Msgpack;

/// A trait for data formats that can be packed and unpacked.
pub trait DataFormat {
    /// The error type for packing.
    type PackError: fmt::Display;
    /// The error type for unpacking.
    type UnpackError: fmt::Display;

    /// Pack data into a byte vector.
    fn pack<D: Serialize>(data: &D) -> Result<Vec<u8>, Self::PackError>;
    /// Unpack data from a byte slice.
    fn unpack<'de, T: Deserialize<'de>>(payload: &'de [u8]) -> Result<T, Self::UnpackError>;
}
