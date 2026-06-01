//! Cliente del servicio de firma de las sedes de La Rioja (carFirma).
//!
//! El servidor expone una API REST bajo `<urlBase>/servicioFirma/firmas`.
//! La URL base llega embebida en la invocación `carfirma://` (campo `id`).

const PATH_RAIZ: &str = "servicioFirma/firmas";
const PATH_STATUS: &str = "status";

pub struct Cliente {
    url_base: String,
}

#[derive(Debug)]
pub struct Estado {
    pub bloqueada: bool,
    pub version_actual: String,
    #[allow(dead_code)]
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

    /// Consulta el estado del servicio. No requiere sesión.
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
