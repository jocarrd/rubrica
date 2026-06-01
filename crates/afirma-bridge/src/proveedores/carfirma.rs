use super::{ProveedorSede, ResultadoFirma, Solicitud};
use crate::larioja;
use crate::protocol::{self, Invocation};

pub struct CarFirmaProvider;

impl ProveedorSede for CarFirmaProvider {
    fn nombre(&self) -> &'static str {
        "carFirma (La Rioja)"
    }

    fn preparar(&self, invocation: &Invocation) -> Solicitud {
        let servidor = invocation
            .id
            .as_deref()
            .and_then(protocol::url_base_from_id)
            .unwrap_or_default();
        let sesion = invocation
            .id
            .as_deref()
            .and_then(protocol::id_from_carfirma_string)
            .unwrap_or_default();

        let documento = if !servidor.is_empty() && !sesion.is_empty() {
            larioja::Cliente::new(&servidor).documento_a_firmar(&sesion)
        } else {
            None
        };

        Solicitud {
            servidor,
            sesion,
            documento,
        }
    }

    fn firmar(&self, solicitud: &Solicitud, identity: &rubrica_core::Identity) -> ResultadoFirma {
        let cliente = larioja::Cliente::new(&solicitud.servidor);
        paso("inicio firma trifásica");

        let Some(doc_id) = cliente.id_documento(&solicitud.sesion) else {
            paso("FALLO: id_documento (sesión caducada o servidor no responde)");
            return ResultadoFirma::error("sesión no válida o caducada");
        };
        paso(&format!("doc_id obtenido: {doc_id}"));

        let cert_der = match identity.cert_der() {
            Ok(d) => d,
            Err(e) => return ResultadoFirma::error(e.to_string()),
        };
        let raiz = identity.root_der();
        paso("enviando pre-firma (POST hash) con el certificado...");

        let hash = match cliente.prefirma(&doc_id, &cert_der, raiz.as_deref()) {
            Ok(h) => h,
            Err(e) => {
                paso(&format!("FALLO en pre-firma: {e}"));
                return ResultadoFirma::error(format!("pre-firma: {e}"));
            }
        };
        paso(&format!("hash recibido del servidor: {} bytes", hash.len()));

        let firma = match identity.firmar_sha256(&hash) {
            Ok(f) => f,
            Err(e) => return ResultadoFirma::error(e.to_string()),
        };
        paso(&format!("firma generada localmente: {} bytes", firma.len()));

        match cliente.finalizar(&doc_id, &firma) {
            Ok(()) => {
                paso("post-firma OK: documento firmado y devuelto a la sede");
                ResultadoFirma::ok("documento firmado y devuelto a la sede")
            }
            Err(e) => {
                paso(&format!("FALLO en post-firma: {e}"));
                ResultadoFirma::error(format!("post-firma: {e}"))
            }
        }
    }
}

fn paso(s: &str) {
    use std::io::Write;
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{home}/.rubrica-firma-larioja.log"))
    {
        let _ = writeln!(f, "{s}");
    }
}

#[allow(dead_code)]
fn guardar_traza(t: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let _ = std::fs::write(format!("{home}/.rubrica-firma-larioja.log"), t);
}
