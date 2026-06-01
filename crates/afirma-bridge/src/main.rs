mod larioja;
mod protocol;

use std::net::TcpListener;
use tungstenite::accept;
use tungstenite::Message;

const ECHO_PREFIX: &str = "echo=";
const ECHO_OK: &str = "OK";
const PORTS: [u16; 5] = [63117, 63118, 63119, 63120, 63121];

fn main() {
    // Modo manejador de protocolo: el sistema nos invoca con la URL carfirma://
    // o afirma:// cuando el usuario pulsa "Firmar" en una sede.
    if let Some(url) = std::env::args().nth(1) {
        if url.starts_with("carfirma://") || url.starts_with("afirma://") {
            handle_url(&url);
            return;
        }
    }
    serve();
}

fn handle_url(url: &str) {
    let invocation = protocol::parse(url);
    let mut report = String::new();
    report.push_str("=== Invocación recibida desde la sede ===\n");
    report.push_str(&format!("URL completa:\n{url}\n\n"));
    report.push_str(&format!("operación: {}\n", invocation.operation));
    if let Some(id) = &invocation.id {
        report.push_str(&format!("id (crudo): {id}\n"));
        if let Some(base) = protocol::url_base_from_id(id) {
            report.push_str(&format!("servidor de firma (urlBase): {base}\n"));
        }
        if let Some(real_id) = protocol::id_from_carfirma_string(id) {
            report.push_str(&format!("id de sesión: {real_id}\n"));
        }
        if let Some(base) = protocol::url_base_from_id(id) {
            match larioja::Cliente::new(&base).estado() {
                Ok(estado) => report.push_str(&format!(
                    "estado del servicio: bloqueada={}, versión={}\n",
                    estado.bloqueada, estado.version_actual
                )),
                Err(e) => report.push_str(&format!("no se pudo contactar el servicio: {e}\n")),
            }
        }
    }
    if let Some(fmt) = &invocation.format {
        report.push_str(&format!("formato: {fmt}\n"));
    }

    print!("{report}");
    log_invocation(&report);
}

fn log_invocation(report: &str) {
    let path = format!("{}/.rubrica-invocacion.log", env_home());
    let _ = std::fs::write(&path, report);
}

fn env_home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn serve() {
    let listener = bind().expect("no se pudo abrir ningún puerto del protocolo afirma");
    let port = listener.local_addr().unwrap().port();
    println!("Puente afirma:// escuchando en 127.0.0.1:{port}");

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        std::thread::spawn(move || {
            if let Ok(mut ws) = accept(stream) {
                while let Ok(msg) = ws.read() {
                    if let Message::Text(text) = msg {
                        let response = handle(&text);
                        let _ = ws.send(Message::Text(response));
                    }
                }
            }
        });
    }
}

fn bind() -> Option<TcpListener> {
    PORTS
        .iter()
        .find_map(|p| TcpListener::bind(("127.0.0.1", *p)).ok())
}

fn handle(message: &str) -> String {
    if let Some(rest) = message.strip_prefix(ECHO_PREFIX) {
        let _ = rest;
        return ECHO_OK.to_string();
    }
    process_operation(message)
}

fn process_operation(message: &str) -> String {
    let invocation = protocol::parse(message);
    format!("RUBRICA_RECEIVED:{}", invocation.operation)
}
