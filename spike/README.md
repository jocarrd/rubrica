# rubrica-spike

## Español

Sprint 0 — *de-risk*. Objetivo único: demostrar que Rust puede generar una firma
**PAdES-B** sobre un PDF que un validador oficial acepte como válida. Si lo
consigue, el proyecto es viable y se levanta el workspace completo; si no, lo
sabemos en días en lugar de en meses.

Esto es código de exploración, no la arquitectura final (ver
[`../docs/es/arquitectura.md`](../docs/es/arquitectura.md)).

### Criterio de continuación (go/no-go)

El PDF de salida debe resultar **válido** en al menos uno de:

- [VALIDe](https://valide.redsara.es/valide/validarFirma/ejecutar.html) — validador
  oficial del Gobierno de España.
- [Demo DSS](https://ec.europa.eu/digital-building-blocks/DSS/webapp-demo/validation)
  — validador de referencia de la Comisión Europea.

### Uso

```bash
openssl genrsa -out key.pem 2048
openssl req -new -x509 -key key.pem -out cert.pem -days 365 -subj "/CN=Rubrica Test"
openssl pkcs12 -export -inkey key.pem -in cert.pem -out spike/fixtures/test.p12 -passout pass:test

cargo run -p rubrica-spike -- \
    --pdf  spike/fixtures/sample.pdf \
    --p12  spike/fixtures/test.p12 \
    --pass test \
    --out  spike/out/sample-signed.pdf
```

---

## English

Sprint 0 — de-risk. Single goal: prove that Rust can generate a **PAdES-B**
signature over a PDF that an official validator accepts as valid. If it does, the
project is viable and we build the full workspace; if not, we learn that in days
rather than months.

This is exploratory code, not the final architecture (see
[`../docs/en/architecture.md`](../docs/en/architecture.md)).

### Go/no-go criterion

The output PDF must be reported **valid** by at least one of:

- [VALIDe](https://valide.redsara.es/valide/validarFirma/ejecutar.html) — the
  Spanish government's official validator.
- [DSS demo](https://ec.europa.eu/digital-building-blocks/DSS/webapp-demo/validation)
  — the European Commission's reference validator.

### Usage

```bash
openssl genrsa -out key.pem 2048
openssl req -new -x509 -key key.pem -out cert.pem -days 365 -subj "/CN=Rubrica Test"
openssl pkcs12 -export -inkey key.pem -in cert.pem -out spike/fixtures/test.p12 -passout pass:test

cargo run -p rubrica-spike -- \
    --pdf  spike/fixtures/sample.pdf \
    --p12  spike/fixtures/test.p12 \
    --pass test \
    --out  spike/out/sample-signed.pdf
```
