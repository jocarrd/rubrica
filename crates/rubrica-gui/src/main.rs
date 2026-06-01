use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rubrica_core::{formats, parse_pkcs12};
use tiny_http::{Header, Method, Response, Server};

const INDEX: &str = include_str!("index.html");

fn main() {
    let addr = "127.0.0.1:8714";
    let server = Server::http(addr).expect("no se pudo abrir el servidor");
    println!("Rúbrica en http://{addr}");

    for mut request in server.incoming_requests() {
        let response = match (request.method(), request.url()) {
            (Method::Get, "/") => html(INDEX),
            (Method::Post, "/api/sign") => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                handle_sign(&body)
            }
            _ => Response::from_string("not found").with_status_code(404),
        };
        let _ = request.respond(response);
    }
}

fn html(content: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(content).with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
    )
}

fn json(body: String, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
}

fn handle_sign(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    match sign(body) {
        Ok(json_body) => json(json_body, 200),
        Err(e) => json(format!("{{\"error\":{}}}", quote(&e)), 400),
    }
}

fn sign(body: &str) -> Result<String, String> {
    let document = field(body, "document").ok_or("falta el documento")?;
    let certificate = field(body, "certificate").ok_or("falta el certificado")?;
    let password = field(body, "password").unwrap_or_default();
    let name = field(body, "name").unwrap_or_else(|| "documento".into());

    let doc_bytes = B64.decode(document).map_err(|_| "documento no es base64")?;
    let cert_bytes = B64
        .decode(certificate)
        .map_err(|_| "certificado no es base64")?;

    let identity = parse_pkcs12_bytes(&cert_bytes, &password)?;

    let signed = if doc_bytes.starts_with(b"%PDF-") {
        formats::pades::sign(&doc_bytes, &identity).map_err(|e| e.to_string())?
    } else {
        formats::cades::sign(&doc_bytes, &identity).map_err(|e| e.to_string())?
    };

    let signer = identity.common_name().unwrap_or_default();
    let filename = signed_name(&name, doc_bytes.starts_with(b"%PDF-"));
    Ok(format!(
        "{{\"signed\":{},\"filename\":{},\"signer\":{}}}",
        quote(&B64.encode(&signed)),
        quote(&filename),
        quote(&signer)
    ))
}

fn parse_pkcs12_bytes(bytes: &[u8], password: &str) -> Result<rubrica_core::Identity, String> {
    let dir = std::env::temp_dir().join(format!("rubrica-cert-{}.p12", std::process::id()));
    std::fs::write(&dir, bytes).map_err(|e| e.to_string())?;
    let result = parse_pkcs12(bytes, password)
        .or_else(|_| rubrica_core::load_pkcs12(&dir, password))
        .map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&dir);
    result
}

fn signed_name(name: &str, is_pdf: bool) -> String {
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    if is_pdf {
        format!("{stem}-firmado.pdf")
    } else {
        format!("{stem}-firmado.p7s")
    }
}

fn field(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = body.find(&needle)? + needle.len();
    let colon = body[start..].find(':')? + start + 1;
    let rest = body[colon..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            _ => out.push(c),
        }
    }
    None
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
