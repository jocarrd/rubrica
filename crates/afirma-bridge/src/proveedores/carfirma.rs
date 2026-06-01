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

        let Some(doc_id) = cliente.id_documento(&solicitud.sesion) else {
            return ResultadoFirma::error("sesión no válida o caducada");
        };
        let cert_der = match identity.cert_der() {
            Ok(d) => d,
            Err(e) => return ResultadoFirma::error(e.to_string()),
        };
        let raiz = identity.root_der();

        // Firma trifásica: la sede da el hash, lo firmamos, lo devolvemos.
        let hash = match cliente.prefirma(&doc_id, &cert_der, raiz.as_deref()) {
            Ok(h) => h,
            Err(e) => return ResultadoFirma::error(e),
        };
        let firma = match identity.firmar_sha256(&hash) {
            Ok(f) => f,
            Err(e) => return ResultadoFirma::error(e.to_string()),
        };
        match cliente.finalizar(&doc_id, &firma) {
            Ok(()) => ResultadoFirma::ok("documento firmado y devuelto a la sede"),
            Err(e) => ResultadoFirma::error(e),
        }
    }
}
