use super::cms;
use super::validate::{verify_detached, Report};
use crate::error::Result;
use crate::keystore::Identity;

pub fn sign(data: &[u8], identity: &Identity) -> Result<Vec<u8>> {
    let digest = cms::sha256(data);
    cms::signed_data_detached(&digest, identity)
}

pub fn verify(data: &[u8], signature: &[u8]) -> Result<Report> {
    verify_detached(data, signature)
}
