<!-- Idioma / Language: **Español** · [English](../en/regional-compatibility.md) -->

# Compatibilidad con las comunidades autónomas

Las sedes electrónicas españolas comparten la plataforma estatal **@firma** y los
formatos estándar (PAdES, CAdES, XAdES). La diferencia entre comunidades está en el
**cliente de escritorio** que la sede invoca al pulsar «Firmar» y en el **protocolo**
con que lo lanza.

## Estado por administración

| Administración | Cliente | Protocolo | Estado en Rúbrica |
|----------------|---------|-----------|-------------------|
| Andalucía, Aragón, Asturias, Baleares, Canarias, Cantabria, Castilla-La Mancha, Castilla y León, Extremadura, Madrid, Murcia, Navarra, Valencia, AEAT, Seguridad Social | AutoFirma estándar | `afirma://` | Cubiertas por el proveedor `afirma` |
| **La Rioja** | carFirma | `carfirma://` | Implementado y verificado |
| **País Vasco** (Izenpe) | Idazki Desktop | `idazki://` | Planificado (mismo patrón que carFirma) |
| **Cataluña** (AOC) | Signador | app propia / AutoFirma | Cubierta en parte vía `afirma://`; app propia a investigar |

Aproximadamente el 83 % de las administraciones usan AutoFirma estándar, por lo que
el proveedor `afirma` ofrece compatibilidad amplia de inmediato. La Rioja, País
Vasco y Cataluña tienen variantes propias que se añaden una a una.

## Arquitectura

El crate `afirma-bridge` define el trait `ProveedorSede`. Cada administración con
protocolo propio es una implementación del trait, seleccionada según el esquema de
la URL de invocación (`carfirma://`, `idazki://`, `afirma://`). Añadir una comunidad
nueva es escribir una implementación pequeña, sin tocar las demás.

```
ProveedorSede (trait)
├─ AfirmaProvider     afirma://    (AutoFirma estándar, ~15 administraciones)
├─ CarFirmaProvider   carfirma://  (La Rioja)
└─ IdazkiProvider     idazki://    (País Vasco)
```

Cada proveedor sabe: interpretar su URL de invocación, descargar el documento a
firmar del servidor de su sede, y devolver el resultado en el formato que esa sede
espera. El núcleo de firma (`rubrica-core`) es común a todos.

### Firma trifásica (La Rioja)

La sede de La Rioja no recibe el documento ya firmado, sino que aplica firma
trifásica del lado del servidor:

1. **Pre-firma:** se envía el certificado del firmante (`POST hash/{id}`) y el
   servidor devuelve el hash a firmar.
2. **Firma:** el cliente firma ese hash con la clave privada del certificado.
3. **Post-firma:** se devuelve la firma (`POST {id}`) y el servidor ensambla y
   guarda el documento firmado.

Por eso `rubrica-core` expone, además de la firma completa de ficheros, la
operación de firmar un hash (`Identity::firmar_sha256`), que usa el proveedor de
La Rioja.

## No bloqueantes

Galicia (Chave365) y, transversalmente, Cl@ve Firma / FIRe son sistemas de **firma en
la nube**: se firman desde el navegador sin cliente de escritorio, así que no
requieren integración local.
