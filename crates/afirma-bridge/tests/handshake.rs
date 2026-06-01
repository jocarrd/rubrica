use std::io::{Read, Write};
use std::net::TcpStream;

#[test]
#[ignore]
fn upgrade_websocket_disponible() {
    let mut stream = TcpStream::connect("127.0.0.1:63117").expect("bridge en marcha");
    let req = "GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n";
    stream.write_all(req.as_bytes()).unwrap();
    let mut buf = [0u8; 256];
    let n = stream.read(&mut buf).unwrap();
    assert!(
        String::from_utf8_lossy(&buf[..n]).contains("101"),
        "debe responder con upgrade WebSocket (101)"
    );
}
