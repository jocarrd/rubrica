import subprocess, socket, base64, struct, time, os, sys

BIN = "/home/anne/Proyectos/rubrica/target/debug/afirma-bridge"

env = dict(os.environ)
env["RUBRICA_NO_ABRIR"] = "1"
proc = subprocess.Popen([BIN], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
time.sleep(2)

def ws_connect(port):
    s = socket.create_connection(("127.0.0.1", port), timeout=3)
    key = base64.b64encode(b"1234567890123456").decode()
    s.send(f"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n".encode())
    s.recv(2048)
    return s

def ws_send(s, msg):
    m = msg.encode(); frame = bytearray([0x81]); ln = len(m); mask = b"\x00\x00\x00\x00"
    if ln < 126: frame.append(0x80 | ln)
    else: frame.append(0x80 | 126); frame += struct.pack(">H", ln)
    frame += mask + m; s.send(frame)

def ws_recv(s):
    h = s.recv(2)
    if len(h) < 2: return "(sin respuesta)"
    ln = h[1] & 0x7f
    if ln == 126: ln = struct.unpack(">H", s.recv(2))[0]
    return s.recv(ln).decode(errors="replace")

try:
    s = ws_connect(63117)
    ws_send(s, "echo=-idsession=ABC123@EOF")
    print("echo      ->", ws_recv(s))
    ws_send(s, "getresult?idsession=ABC123")
    print("getresult ->", ws_recv(s))
    # operación de firma con un PDF mínimo en base64 (%PDF-1.4)
    pdf_b64 = base64.b64encode(b"%PDF-1.4\n").decode()
    ws_send(s, f"afirma://sign?op=sign&format=pades&algorithm=SHA256withRSA&dat={pdf_b64}")
    print("operacion ->", ws_recv(s))
except Exception as e:
    print("EXCEPCION:", repr(e))
finally:
    proc.terminate()
