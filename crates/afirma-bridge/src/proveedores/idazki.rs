use super::{ProveedorSede, ResultadoFirma, Solicitud};
use crate::protocol::Invocation;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub struct IdazkiProvider;

impl ProveedorSede for IdazkiProvider {
    fn nombre(&self) -> &'static str {
        "Idazki (País Vasco)"
    }

    fn preparar(&self, invocation: &Invocation) -> Solicitud {
        let documento = invocation
            .data
            .as_deref()
            .and_then(|d| STANDARD.decode(d).ok());
        Solicitud {
            servidor: invocation.id.clone().unwrap_or_default(),
            sesion: invocation.id.clone().unwrap_or_default(),
            documento,
        }
    }

    fn firmar(&self, solicitud: &Solicitud, identity: &rubrica_core::Identity) -> ResultadoFirma {
        super::afirma::firmar_local(solicitud, identity)
    }
}
