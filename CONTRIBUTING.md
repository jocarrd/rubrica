# Contributing · Contribuir

## Español

Gracias por tu interés en Rúbrica.

### Entorno

- Rust estable (instalable con [rustup](https://rustup.rs)).
- Para el DNIe y tarjetas criptográficas: `opensc` y `pcscd`.

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

### Pautas

- El código debe pasar `cargo fmt` y `cargo clippy` sin avisos.
- Toda la lógica de firma vive en el núcleo (`rubrica-core`); la GUI y la CLI son
  capas finas. Acompaña los cambios de firma con tests.
- Las firmas se validan contra herramientas oficiales (VALIDe, demo DSS) y vectores
  conocidos antes de darse por buenas.
- Línea legal: el protocolo de las sedes se documenta desde fuentes públicas y
  capturas de red. No se porta código de `clienteafirma` (GPL/EUPL).

### Seguridad

No abras una incidencia pública para vulnerabilidades. Escribe en privado al autor.
Nunca subas claves privadas ni certificados reales al repositorio.

---

## English

Thanks for your interest in Rúbrica.

### Environment

- Stable Rust (install via [rustup](https://rustup.rs)).
- For the DNIe and smart cards: `opensc` and `pcscd`.

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

### Guidelines

- Code must pass `cargo fmt` and `cargo clippy` with no warnings.
- All signing logic lives in the core (`rubrica-core`); the GUI and CLI are thin
  layers. Pair signing changes with tests.
- Signatures are validated against official tools (VALIDe, DSS demo) and known
  vectors before being accepted.
- Legal line: the portal protocol is documented from public sources and network
  captures. We do not port code from `clienteafirma` (GPL/EUPL).

### Security

Do not open a public issue for vulnerabilities. Contact the author privately. Never
commit private keys or real certificates.
