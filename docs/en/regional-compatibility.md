<!-- Language / Idioma: **English** · [Español](../es/compatibilidad-ccaa.md) -->

# Compatibility with the autonomous communities

Spanish public-administration portals share the state-wide **@firma** platform and
the standard formats (PAdES, CAdES, XAdES). What differs between regions is the
**desktop client** the portal invokes when you click "Sign" and the **protocol** it
uses to launch it.

## Status per administration

| Administration | Client | Protocol | Status in Rúbrica |
|----------------|--------|----------|-------------------|
| Andalucía, Aragón, Asturias, Baleares, Canarias, Cantabria, Castilla-La Mancha, Castilla y León, Extremadura, Madrid, Murcia, Navarra, Valencia, AEAT, Social Security | Standard AutoFirma | `afirma://` | Covered by the `afirma` provider |
| **La Rioja** | carFirma | `carfirma://` | Implemented and verified |
| **Basque Country** (Izenpe) | Idazki Desktop | `idazki://` | Planned (same pattern as carFirma) |
| **Catalonia** (AOC) | Signador | own app / AutoFirma | Partly covered via `afirma://`; own app to investigate |

About 83% of administrations use standard AutoFirma, so the `afirma` provider gives
broad compatibility right away. La Rioja, the Basque Country and Catalonia have their
own variants, added one by one.

## Architecture

The `afirma-bridge` crate defines the `ProveedorSede` (portal provider) trait. Each
administration with its own protocol is an implementation of the trait, selected by
the scheme of the invocation URL (`carfirma://`, `idazki://`, `afirma://`). Adding a
new region means writing a small implementation, without touching the others.

```
ProveedorSede (trait)
├─ AfirmaProvider     afirma://    (standard AutoFirma, ~15 administrations)
├─ CarFirmaProvider   carfirma://  (La Rioja)
└─ IdazkiProvider     idazki://    (Basque Country)
```

Each provider knows how to: parse its invocation URL, download the document to be
signed from its portal's server, and return the result in the format that portal
expects. The signing core (`rubrica-core`) is shared by all.

## Non-blocking

Galicia (Chave365) and, across the board, Cl@ve Firma / FIRe are **cloud-signing**
systems: signed from the browser with no desktop client, so they need no local
integration.
