use serde::{Deserialize, Serialize};

use super::DataFormat;

/// JSON data format packer.
pub struct Packer;

impl DataFormat for Packer {
    type PackError = serde_json::Error;
    type UnpackError = serde_json::Error;

    fn pack<D: Serialize>(data: &D) -> Result<Vec<u8>, Self::PackError> {
        serde_json::to_vec(data)
    }

    fn unpack<'de, T: Deserialize<'de>>(payload: &'de [u8]) -> Result<T, Self::UnpackError> {
        serde_json::from_slice(payload)
    }
}
