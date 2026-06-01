#!/bin/bash
# Registra Rúbrica como manejador de los protocolos carfirma:// y afirma://
# de las sedes electrónicas, sustituyendo a AutoFirma / carFirma.
set -e

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
mkdir -p "$BIN_DIR" "$APP_DIR"

SOURCE_BIN="$(dirname "$0")/../../target/release/afirma-bridge"
if [ ! -f "$SOURCE_BIN" ]; then
    SOURCE_BIN="$(dirname "$0")/../../target/debug/afirma-bridge"
fi
install -m 755 "$SOURCE_BIN" "$BIN_DIR/rubrica-bridge"

cat > "$APP_DIR/rubrica.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Rúbrica
Comment=Firma electrónica para las sedes españolas
Exec=${BIN_DIR}/rubrica-bridge %u
Icon=rubrica
Terminal=false
Categories=Office;Security;
MimeType=x-scheme-handler/carfirma;x-scheme-handler/afirma;x-scheme-handler/idazki;
StartupNotify=false
EOF

update-desktop-database "$APP_DIR" 2>/dev/null || true
xdg-mime default rubrica.desktop x-scheme-handler/carfirma
xdg-mime default rubrica.desktop x-scheme-handler/afirma
xdg-mime default rubrica.desktop x-scheme-handler/idazki

echo "Rúbrica registrada como manejador de:"
echo "  carfirma:// -> $(xdg-mime query default x-scheme-handler/carfirma)"
echo "  afirma://   -> $(xdg-mime query default x-scheme-handler/afirma)"
echo "  idazki://   -> $(xdg-mime query default x-scheme-handler/idazki)"
