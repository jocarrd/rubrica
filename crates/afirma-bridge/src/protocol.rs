use std::collections::HashMap;

#[derive(Debug, Default, PartialEq)]
pub struct Invocation {
    pub operation: String,
    pub id: Option<String>,
    pub keystore: Option<String>,
    pub keystore_lib: Option<String>,
    pub single_cert: bool,
    pub only_dnie: bool,
    pub only_sign: bool,
    pub no_expired: bool,
    pub filters: Option<String>,
    pub format: Option<String>,
    pub algorithm: Option<String>,
    pub data: Option<String>,
}

pub fn parse(uri: &str) -> Invocation {
    let without_scheme = uri.split_once("://").map(|x| x.1).unwrap_or(uri);
    let (operation, query) = match without_scheme.split_once('?') {
        Some((op, q)) => (op.trim_end_matches('/'), q),
        None => (without_scheme.trim_end_matches('/'), ""),
    };

    let params = parse_query(query);
    Invocation {
        operation: operation.to_string(),
        id: params.get("id").cloned(),
        keystore: params.get("ks").cloned(),
        keystore_lib: params.get("ksl").cloned(),
        single_cert: params.contains_key("un"),
        only_dnie: params.contains_key("dn"),
        only_sign: params.contains_key("sf"),
        no_expired: params.contains_key("nc"),
        filters: params.get("fc").cloned(),
        format: params.get("format").cloned(),
        algorithm: params.get("algorithm").cloned(),
        data: params.get("dat").cloned(),
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    if query.is_empty() {
        return params;
    }
    for pair in query.split('&') {
        match pair.split_once('=') {
            Some((k, v)) => {
                params.insert(k.to_string(), url_decode(v));
            }
            None => {
                params.insert(pair.to_string(), String::new());
            }
        }
    }
    params
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let Some(byte) = from_hex(bytes[i + 1], bytes[i + 2]) {
                    out.push(byte);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(hi: u8, lo: u8) -> Option<u8> {
    let h = (hi as char).to_digit(16)?;
    let l = (lo as char).to_digit(16)?;
    Some((h * 16 + l) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_operacion_y_parametros() {
        let uri = "afirma://sign?op=sign&format=pades&algorithm=SHA256withRSA&ks=WINDOWS&un&dat=SGVsbG8%3D";
        let inv = parse(uri);
        assert_eq!(inv.operation, "sign");
        assert_eq!(inv.format.as_deref(), Some("pades"));
        assert_eq!(inv.algorithm.as_deref(), Some("SHA256withRSA"));
        assert_eq!(inv.keystore.as_deref(), Some("WINDOWS"));
        assert!(inv.single_cert);
        assert_eq!(inv.data.as_deref(), Some("SGVsbG8="));
    }

    #[test]
    fn parsea_flags_de_certificado() {
        let inv = parse("afirma://sign?id=ABC123&dn&sf&nc");
        assert_eq!(inv.id.as_deref(), Some("ABC123"));
        assert!(inv.only_dnie);
        assert!(inv.only_sign);
        assert!(inv.no_expired);
        assert!(!inv.single_cert);
    }

    #[test]
    fn sin_parametros() {
        let inv = parse("afirma://echo");
        assert_eq!(inv.operation, "echo");
        assert!(inv.id.is_none());
    }

    #[test]
    fn url_decode_porcentajes() {
        assert_eq!(url_decode("a%20b%2Bc"), "a b+c");
        assert_eq!(url_decode("https%3A%2F%2Fx.es"), "https://x.es");
    }
}
