use serde::{Deserialize, Serialize};

use super::DataFormat;

/// MessagePack data format packer.
pub struct Packer;

impl DataFormat for Packer {
    type PackError = rmp_serde::encode::Error;
    type UnpackError = rmp_serde::decode::Error;

    fn pack<D: Serialize>(data: &D) -> Result<Vec<u8>, Self::PackError> {
        rmp_serde::to_vec_named(data)
    }

    fn unpack<'de, T: Deserialize<'de>>(payload: &'de [u8]) -> Result<T, Self::UnpackError> {
        rmp_serde::from_slice(payload)
    }
}
