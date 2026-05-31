<!-- Language / Idioma: **English** · [Español](../es/arquitectura.md) -->

# Architecture

> A modern electronic-signature client compatible with Spanish
> public-administration portals (the @firma platform), built to work well where
> AutoFirma struggles.
>
> - **Stack:** native Rust, no runtime. Native GUI (egui / slint / gtk-rs).
> - **Initial platform:** Linux.
> - **License:** MIT or Apache-2.0. Permissive because we do not derive from
>   `clienteafirma` (GPL/EUPL); the engine is built from permissive crates or
>   integrated as a separate process.

## The idea in one sentence

We do not reinvent signature cryptography (open ETSI standards); we reinvent the
experience, the packaging and the browser integration, which is where the real
pain is.

## Signing engine

Choosing native Rust means giving up DSS (which is Java), so the signing engine —
not the UI — is the hard part. A realistic plan per format:

| Format | Portal usage | Difficulty in Rust | Plan |
|--------|--------------|--------------------|------|
| CAdES (CMS/PKCS#7) | High | Medium | MVP. `cms` + `x509-cert` (RustCrypto) |
| PAdES (PDF) | Very high | Medium-high | MVP. Detached CAdES embedded via `lopdf` |
| XAdES (XML-DSig) | Very high | High (canonicalization) | Phase 2. Bindings to `xmlsec`/`libxml2` or a sidecar |
| ASiC | Medium | Medium | Phase 3 (ZIP container over the above) |

XAdES is Rust's Achilles heel: XML canonicalization (C14N) and XML-DSig have no
mature crate. It is the only part that may need a sidecar or C bindings, but it
does not block the MVP.

Signature levels (ETSI profiles): the MVP covers **-B** (baseline) and **-T**
(with an RFC 3161 timestamp). Levels -LT and -LTA (embedded OCSP/CRL, long-term
archival) come later.

## Certificate and key access

A common `SignerProvider` trait with three implementations:

1. **PKCS#11** — DNIe and smart cards, via the `cryptoki` crate pointing at
   `opensc-pkcs11.so`. Requires `opensc` and `pcscd`. PIN handling.
2. **PKCS#12** — `.p12`/`.pfx` files, such as an exported FNMT certificate.
3. **System NSS store** — `~/.pki/nssdb`, the one Firefox and Chrome use on Linux.
   (Native 64-bit, without the architecture problem carFirma had.)

## Workspace layout

```
rubrica/
├─ spike/                 de-risk: sign PAdES-B and validate (current phase)
├─ crates/
│  ├─ rubrica-core/       pure logic, testable in isolation
│  │  ├─ keystore/        SignerProvider: pkcs11, pkcs12, nss
│  │  ├─ formats/         cades, pades, (xades in phase 2)
│  │  ├─ tsa/             RFC 3161 timestamping
│  │  └─ validation/      signature verification
│  ├─ rubrica-cli/        command-line interface over the core
│  └─ afirma-bridge/      phase 3: local server + afirma:// handler
├─ app/                   native GUI
└─ packaging/             AppImage + Flatpak
```

Golden rule: all logic lives in `rubrica-core`, a standalone crate with tests. The
GUI and CLI are thin layers on top. This gives us a CLI from day one reusing the
same core.

## Portal integration (phase 3)

Portals do not call the app directly: their JavaScript (`autoscript.js`) invokes
the AutoFirma protocol, which has two mechanisms:

- it registers the `afirma://` scheme (on Linux through a `.desktop` file with
  `MimeType=x-scheme-handler/afirma`, the same mechanism carFirma used with
  `x-scheme-handler/carfirma`), and
- it starts a local server on `127.0.0.1` (ports around 63117) speaking a JSON
  protocol with commands (`sign`, `cosign`, `batch`; base64 parameters).

Being compatible with every region means implementing that protocol exactly. It is
understood from the public protocol and network captures. We document it from
public sources and observation; we do not port GPL code.

## Packaging

- **AppImage** (a single executable file) and **Flatpak** (GNOME Software and
  derivatives).
- Native 64-bit binary, no 32-bit dependencies. The DNIe PKCS#11 modules are taken
  from the system (`opensc`), with the dependency documented.
- Registration of the `afirma://` handler at install time.

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| XAdES/C14N in Rust is hard | Defer to phase 2; sidecar or xmlsec bindings |
| The portal protocol changes | Isolate it in `afirma-bridge`; integration tests |
| Trust (legal validity) | Validate against official tools and known vectors |
| Abandoning midway | Ship something useful in phase 1; don't do everything at once |
| Clean-room legal line | Document from public sources, do not port GPL code |
