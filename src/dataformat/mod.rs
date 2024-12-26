use core::fmt;

use serde::{Deserialize, Serialize};

mod json;
pub use json::Packer as Json;
#[cfg(feature = "msgpack")]
mod msgpack;
#[cfg(feature = "msgpack")]
pub use msgpack::Packer as Msgpack;

pub trait DataFormat {
    type PackError: fmt::Display;
    type UnpackError: fmt::Display;

    fn pack<D: Serialize>(data: &D) -> Result<Vec<u8>, Self::PackError>;
    fn unpack<'de, T: Deserialize<'de>>(payload: &'de [u8]) -> Result<T, Self::UnpackError>;
}
