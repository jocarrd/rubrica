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
            let cliente = larioja::Cliente::new(&base);
            match cliente.estado() {
                Ok(estado) => report.push_str(&format!(
                    "estado del servicio: bloqueada={}, versión={}\n",
                    estado.bloqueada, estado.version_actual
                )),
                Err(e) => report.push_str(&format!("no se pudo contactar el servicio: {e}\n")),
            }
            // Captura del protocolo con la sesión viva (diagnóstico e2e).
            if let Some(sesion) = protocol::id_from_carfirma_string(id) {
                report.push_str("\n=== Sondeo de la sesión (respuestas crudas del servidor) ===");
                report.push_str(&cliente.sondear_sesion(&sesion));
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

    let documento = invocation
        .id
        .as_deref()
        .and_then(protocol::url_base_from_id)
        .zip(sesion_no_vacia(&sesion))
        .and_then(|(base, ses)| larioja::Cliente::new(&base).documento_a_firmar(&ses));

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let req = leer_peticion(&mut stream);
        if req.starts_with("POST /firmar") {
            let cuerpo = req.rsplit("\r\n\r\n").next().unwrap_or("");
            let resultado = firmar_solicitud(cuerpo, documento.as_deref());
            responder(&mut stream, "application/json", &resultado);
            break;
        } else {
            responder(&mut stream, "text/html; charset=utf-8", &page);
        }
    }
}

fn sesion_no_vacia(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn leer_peticion(stream: &mut std::net::TcpStream) -> String {
    use std::io::Read;
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

fn responder(stream: &mut std::net::TcpStream, tipo: &str, cuerpo: &str) {
    use std::io::Write;
    let _ = stream.write_all(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {tipo}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
            cuerpo.len()
        )
        .as_bytes(),
    );
}

fn firmar_solicitud(cuerpo_json: &str, documento: Option<&[u8]>) -> String {
    let cert = protocol::json_campo(cuerpo_json, "certificado").unwrap_or_default();
    let pass = protocol::json_campo(cuerpo_json, "password").unwrap_or_default();

    let Some(pdf) = documento else {
        return "{\"ok\":false,\"error\":\"no se pudo descargar el documento de la sede\"}".into();
    };
    let identity = match rubrica_core::load_pkcs12(std::path::Path::new(&cert), &pass) {
        Ok(id) => id,
        Err(e) => {
            return format!(
                "{{\"ok\":false,\"error\":\"{}\"}}",
                escape_json(&e.to_string())
            )
        }
    };
    match rubrica_core::formats::pades::sign(pdf, &identity) {
        Ok(firmado) => {
            let salida = format!("{}/.rubrica-firmado.pdf", env_home());
            let _ = std::fs::write(&salida, &firmado);
            format!("{{\"ok\":true,\"bytes\":{}}}", firmado.len())
        }
        Err(e) => format!(
            "{{\"ok\":false,\"error\":\"{}\"}}",
            escape_json(&e.to_string())
        ),
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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
        return "<div class=\"empty\">No se encontró ningún certificado (.p12/.pfx) en tu carpeta personal ni en Descargas.</div>".into();
    }
    certs
        .iter()
        .map(|c| {
            format!(
                "<div class=\"cert\" data-ruta=\"{}\">\
<span class=\"ico\">🔑</span>\
<span class=\"nm\">{}</span>\
<span class=\"ck\">✓</span>\
</div>",
                escape(&c.ruta.to_string_lossy()),
                escape(&nombre_legible(&c.nombre))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn nombre_legible(archivo: &str) -> String {
    archivo
        .trim_end_matches(".p12")
        .trim_end_matches(".pfx")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
