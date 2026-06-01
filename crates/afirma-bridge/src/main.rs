mod certificados;
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
    mostrar_ventana(&invocation, &report);
}

fn mostrar_ventana(invocation: &protocol::Invocation, report: &str) {
    let servidor = invocation
        .id
        .as_deref()
        .and_then(protocol::url_base_from_id)
        .unwrap_or_else(|| "(desconocido)".into());
    let sesion = invocation
        .id
        .as_deref()
        .and_then(protocol::id_from_carfirma_string)
        .unwrap_or_default();

    let page = ventana_html(&invocation.operation, &servidor, &sesion, report);
    let addr = std::env::var("RUBRICA_VENTANA_ADDR").unwrap_or_else(|_| "127.0.0.1:0".into());
    let Ok(listener) = TcpListener::bind(addr) else {
        return;
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    abrir_navegador(&format!("http://127.0.0.1:{port}/"));

    // Servimos la página a la primera (y única) visita y terminamos.
    if let Some(Ok(mut stream)) = listener.incoming().next() {
        use std::io::Write;
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                page.len(),
                page
            )
            .as_bytes(),
        );
    }
}

fn abrir_navegador(url: &str) {
    if std::env::var("RUBRICA_NO_ABRIR").is_ok() {
        return;
    }
    // Abrimos una ventana de aplicación dedicada (sin pestañas ni barra de
    // direcciones), de modo que parezca una ventana nativa de Rúbrica.
    let navegadores = [
        "chromium",
        "google-chrome",
        "chromium-browser",
        "brave-browser",
    ];
    let geometria = "--app=";
    for nav in navegadores {
        let lanzado = std::process::Command::new(nav)
            .arg(format!("{geometria}{url}"))
            .arg("--window-size=560,640")
            .spawn()
            .is_ok();
        if lanzado {
            return;
        }
    }
    // Si no hay navegador con modo app, recurrimos al manejador del sistema.
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn ventana_html(operacion: &str, servidor: &str, sesion: &str, report: &str) -> String {
    let plantilla = include_str!("ventana.html");
    plantilla
        .replace("{{OPERACION}}", &escape(operacion))
        .replace("{{SERVIDOR}}", &escape(servidor))
        .replace("{{SESION}}", &escape(sesion))
        .replace("{{OPCIONES_CERT}}", &opciones_certificados())
        .replace("{{DETALLE}}", &escape(report))
}

fn opciones_certificados() -> String {
    let certs = certificados::disponibles();
    if certs.is_empty() {
        return "<option value=\"\">No se encontró ningún certificado (.p12/.pfx)</option>".into();
    }
    certs
        .iter()
        .map(|c| {
            format!(
                "<option value=\"{}\">{}</option>",
                escape(&c.ruta.to_string_lossy()),
                escape(&c.nombre)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
