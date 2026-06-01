//! Cliente del servicio de firma de las sedes de La Rioja (carFirma).
//!
//! El servidor expone una API REST bajo `<urlBase>/servicioFirma/firmas`.
//! La URL base llega embebida en la invocación `carfirma://` (campo `id`).

const PATH_RAIZ: &str = "servicioFirma/firmas";
#[allow(dead_code)]
const PATH_STATUS: &str = "status";

pub struct Cliente {
    url_base: String,
}

// Diagnóstico del servicio: se conserva para inspeccionar el estado y el
// protocolo de una sesión real cuando se depura la integración.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Estado {
    pub bloqueada: bool,
    pub version_actual: String,
    pub version_minima: String,
}

impl Cliente {
    pub fn new(url_base: &str) -> Self {
        Self {
            url_base: url_base.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{PATH_RAIZ}/{path}", self.url_base)
    }

    /// Descarga el PDF a firmar siguiendo el flujo real de la sede:
    /// `origen/{sesion}` da los metadatos (incluido el id del documento),
    /// y `origen/{idDocumento}` devuelve el PDF en base64.
    pub fn documento_a_firmar(&self, sesion: &str) -> Option<Vec<u8>> {
        let meta_b64 = contenido_de(&self.url(&format!("origen/{sesion}")))?;
        let meta = decode_b64(&meta_b64)?;
        let meta_txt = String::from_utf8_lossy(&meta);
        let doc_id = meta_txt.rsplit('|').next()?.trim();
        if doc_id.is_empty() {
            return None;
        }
        let pdf_b64 = contenido_de(&self.url(&format!("origen/{doc_id}")))?;
        decode_b64(&pdf_b64)
    }

    /// Consulta el estado del servicio. No requiere sesión.
    #[allow(dead_code)]
    pub fn estado(&self) -> Result<Estado, String> {
        let url = self.url(PATH_STATUS);
        let response = minreq::get(&url)
            .with_header("Accept", "application/json")
            .with_timeout(15)
            .send()
            .map_err(|e| e.to_string())?;
        if response.status_code != 200 {
            return Err(format!("el servicio respondió {}", response.status_code));
        }
        let body = response.as_str().map_err(|e| e.to_string())?;
        Ok(parse_estado(body))
    }

    /// Sondea, con un id de sesión vivo, los endpoints de obtención de datos
    /// del servidor y devuelve un informe con las respuestas crudas. Sirve para
    /// capturar el formato real del protocolo durante una firma en la sede.
    #[allow(dead_code)]
    pub fn sondear_sesion(&self, id: &str) -> String {
        let mut log = String::new();
        let pruebas = [
            ("GET", format!("{id}/origen")),
            ("GET", format!("origen/{id}")),
            ("POST", format!("hash/{id}")),
            ("GET", id.to_string()),
            ("GET", format!("config/{id}")),
        ];
        for (metodo, path) in &pruebas {
            let url = self.url(path);
            let resultado = match *metodo {
                "POST" => minreq::post(&url)
                    .with_header("Accept", "application/json")
                    .with_header("Content-Type", "application/json")
                    .with_body("{}")
                    .with_timeout(15)
                    .send(),
                _ => minreq::get(&url)
                    .with_header("Accept", "application/json")
                    .with_timeout(15)
                    .send(),
            };
            log.push_str(&format!("\n--- {metodo} {url} ---\n"));
            match resultado {
                Ok(r) => {
                    let cuerpo = r.as_str().unwrap_or("(binario)");
                    let muestra: String = cuerpo.chars().take(600).collect();
                    log.push_str(&format!("HTTP {}\n{muestra}\n", r.status_code));
                }
                Err(e) => log.push_str(&format!("error: {e}\n")),
            }
        }
        log
    }
}

fn parse_estado(json: &str) -> Estado {
    Estado {
        bloqueada: campo(json, "bloqueada") == Some("true".to_string()),
        version_actual: campo(json, "versionActual").unwrap_or_default(),
        version_minima: campo(json, "versionMinimaPermitida").unwrap_or_default(),
    }
}

fn campo(json: &str, clave: &str) -> Option<String> {
    let needle = format!("\"{clave}\"");
    let start = json.find(&needle)? + needle.len();
    let colon = json[start..].find(':')? + start + 1;
    let rest = json[colon..].trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn contenido_de(url: &str) -> Option<String> {
    let r = minreq::get(url)
        .with_header("Accept", "application/json")
        .with_timeout(20)
        .send()
        .ok()?;
    if r.status_code != 200 {
        return None;
    }
    campo(r.as_str().ok()?, "contenido")
}

fn decode_b64(s: &str) -> Option<Vec<u8>> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.decode(s).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construye_la_url_del_servicio() {
        let c = Cliente::new("https://ias1.larioja.org/carFirma");
        assert_eq!(
            c.url("status"),
            "https://ias1.larioja.org/carFirma/servicioFirma/firmas/status"
        );
    }

    #[test]
    fn parsea_el_estado() {
        let json = r#"{"bloqueada":"false","urlActualizacion":"x","versionActual":"1","versionMinimaPermitida":"1"}"#;
        let e = parse_estado(json);
        assert!(!e.bloqueada);
        assert_eq!(e.version_actual, "1");
        assert_eq!(e.version_minima, "1");
    }
}
