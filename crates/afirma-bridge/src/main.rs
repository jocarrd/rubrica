use std::net::TcpListener;
use tungstenite::accept;
use tungstenite::Message;

const ECHO_PREFIX: &str = "echo=";
const ECHO_OK: &str = "OK";
const PORTS: [u16; 5] = [63117, 63118, 63119, 63120, 63121];

fn main() {
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
    let op = message.split('&').next().unwrap_or("");
    format!("RUBRICA_RECEIVED:{op}")
}
