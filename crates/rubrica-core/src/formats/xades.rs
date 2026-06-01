use crate::error::{Error, Result};
use crate::keystore::Identity;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use der::Encode;
use rsa::pkcs1v15::SigningKey;
use rsa::sha2::Sha256;
use rsa::signature::{SignatureEncoding, Signer};
use sha2::{Digest, Sha256 as ShaHash};

const DSIG: &str = "http://www.w3.org/2000/09/xmldsig#";
const C14N: &str = "http://www.w3.org/TR/2001/REC-xml-c14n-20010315";
const SIG_RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";

pub fn sign(data: &[u8], identity: &Identity) -> Result<Vec<u8>> {
    let object = format!(
        "<Object xmlns=\"{DSIG}\" Id=\"obj\">{}</Object>",
        B64.encode(data)
    );
    let object_digest = B64.encode(ShaHash::digest(object.as_bytes()));

    let signed_info = format!(
        "<SignedInfo xmlns=\"{DSIG}\">\
<CanonicalizationMethod Algorithm=\"{C14N}\"></CanonicalizationMethod>\
<SignatureMethod Algorithm=\"{SIG_RSA_SHA256}\"></SignatureMethod>\
<Reference URI=\"#obj\">\
<DigestMethod Algorithm=\"{SHA256}\"></DigestMethod>\
<DigestValue>{object_digest}</DigestValue>\
</Reference>\
</SignedInfo>"
    );

    let signing_key = SigningKey::<Sha256>::new(identity.key.clone());
    let signature = signing_key.sign(signed_info.as_bytes());
    let signature_b64 = B64.encode(signature.to_bytes());

    let cert_der = identity
        .cert
        .to_der()
        .map_err(|e| Error::Crypto(e.to_string()))?;
    let cert_b64 = B64.encode(cert_der);

    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Signature xmlns=\"{DSIG}\">\
{signed_info}\
<SignatureValue>{signature_b64}</SignatureValue>\
<KeyInfo><X509Data><X509Certificate>{cert_b64}</X509Certificate></X509Data></KeyInfo>\
<Object xmlns=\"{DSIG}\" Id=\"obj\">{}</Object>\
</Signature>",
        B64.encode(data)
    );

    Ok(xml.into_bytes())
}
