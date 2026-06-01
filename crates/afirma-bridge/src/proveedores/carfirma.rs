//! Proveedor de La Rioja (carFirma, protocolo `carfirma://`).
//!
//! Verificado contra el servidor real https://ias1.larioja.org/carFirma.

use super::{ProveedorSede, Solicitud};
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
}
