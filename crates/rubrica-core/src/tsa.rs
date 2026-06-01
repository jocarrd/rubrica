use crate::error::{Error, Result};
use der::asn1::{Int, ObjectIdentifier, OctetString};
use der::{Decode, Encode};
use sha2::{Digest, Sha256};

const OID_SHA256: &str = "2.16.840.1.101.3.4.2.1";
const DEFAULT_TSA: &str = "https://freetsa.org/tsr";

#[derive(der::Sequence)]
struct AlgorithmIdentifier {
    algorithm: ObjectIdentifier,
}

#[derive(der::Sequence)]
struct MessageImprint {
    hash_algorithm: AlgorithmIdentifier,
    hashed_message: OctetString,
}

#[derive(der::Sequence)]
struct TimeStampReq {
    version: Int,
    message_imprint: MessageImprint,
    cert_req: bool,
}

pub fn timestamp_token(data: &[u8], url: Option<&str>) -> Result<Vec<u8>> {
    let url = url.unwrap_or(DEFAULT_TSA);
    let digest = Sha256::digest(data);

    let request = TimeStampReq {
        version: Int::new(&[1]).map_err(|e| Error::Crypto(e.to_string()))?,
        message_imprint: MessageImprint {
            hash_algorithm: AlgorithmIdentifier {
                algorithm: ObjectIdentifier::new(OID_SHA256)
                    .map_err(|e| Error::Crypto(e.to_string()))?,
            },
            hashed_message: OctetString::new(digest.to_vec())
                .map_err(|e| Error::Crypto(e.to_string()))?,
        },
        cert_req: true,
    };
    let body = request.to_der().map_err(|e| Error::Crypto(e.to_string()))?;

    let response = minreq::post(url)
        .with_header("Content-Type", "application/timestamp-query")
        .with_body(body)
        .with_timeout(20)
        .send()
        .map_err(|e| Error::Tsa(e.to_string()))?;

    if response.status_code != 200 {
        return Err(Error::Tsa(format!(
            "la TSA respondió {}",
            response.status_code
        )));
    }

    extract_token(response.as_bytes())
}

fn extract_token(resp: &[u8]) -> Result<Vec<u8>> {
    let response = TimeStampResp::from_der(resp)
        .map_err(|e| Error::Tsa(format!("respuesta inválida: {e}")))?;
    if response.status.status != 0 && response.status.status != 1 {
        return Err(Error::Tsa(format!(
            "la TSA rechazó la petición (estado {})",
            response.status.status
        )));
    }
    response
        .time_stamp_token
        .ok_or_else(|| Error::Tsa("la respuesta no incluye sello".into()))?
        .to_der()
        .map_err(|e| Error::Tsa(e.to_string()))
}

#[derive(der::Sequence)]
struct PkiStatusInfo {
    status: u32,
}

#[derive(der::Sequence)]
struct TimeStampResp {
    status: PkiStatusInfo,
    #[asn1(optional = "true")]
    time_stamp_token: Option<der::Any>,
}
