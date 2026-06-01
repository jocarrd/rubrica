use super::cms;
use crate::error::Result;
use crate::keystore::Identity;

pub fn sign(data: &[u8], identity: &Identity) -> Result<Vec<u8>> {
    let digest = cms::sha256(data);
    cms::signed_data_detached(&digest, identity)
}
